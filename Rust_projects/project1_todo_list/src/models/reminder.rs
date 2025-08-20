use chrono::{NaiveDate, Datelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CounterType {
    Water,
    Electricity,
}

impl CounterType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CounterType::Water => "water",
            CounterType::Electricity => "electricity",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CounterType::Water => "💧 Вода",
            CounterType::Electricity => "⚡ Электричество",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "water" => Some(CounterType::Water),
            "electricity" => Some(CounterType::Electricity),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CounterReminder {
    pub counter_type: CounterType,
    pub start_day: u32,  // День месяца начала периода (1-31)
    pub end_day: u32,    // День месяца окончания периода (1-31)
    pub enabled: bool,
    pub last_sent_month: Option<String>, // "2024-01" формат для отслеживания
    #[serde(default)]
    pub last_sent_date: Option<String>, // "2024-01-15" для защиты от дублей в сутки
    pub completed_this_month: bool,
}

impl CounterReminder {
    pub fn new(counter_type: CounterType, start_day: u32, end_day: u32) -> Self {
        Self {
            counter_type,
            start_day,
            end_day,
            enabled: true,
            last_sent_month: None,
            last_sent_date: None,
            completed_this_month: false,
        }
    }

    pub fn should_remind_today(&self, today: NaiveDate) -> bool {
        if !self.enabled || self.completed_this_month {
            return false;
        }

        let day = today.day();
        let current_month = today.format("%Y-%m").to_string();
        
        // Сбрасываем статус завершения для нового месяца
        if let Some(ref last_month) = self.last_sent_month {
            if last_month != &current_month {
                // Новый месяц - сбрасываем completed_this_month
                // Это должно делаться в другом месте, но пока оставим здесь
            }
        }

        // Проверяем, находимся ли мы в периоде подачи показаний
        if day < self.start_day || day > self.end_day {
            return false;
        }

        // Логика напоминаний:
        // 1. В первый день периода
        // 2. В середине периода  
        // 3. Каждый день за последние 3 дня
        
        if day == self.start_day {
            return true; // Первый день
        }

        let middle_day = (self.start_day + self.end_day) / 2;
        if day == middle_day {
            return true; // Середина периода
        }

        // Последние 3 дня периода
        if day > self.end_day.saturating_sub(3) {
            return true;
        }

        false
    }

    pub fn mark_sent(&mut self, date: NaiveDate) {
        self.last_sent_month = Some(date.format("%Y-%m").to_string());
        self.last_sent_date = Some(date.format("%Y-%m-%d").to_string());
    }

    pub fn mark_completed(&mut self) {
        self.completed_this_month = true;
    }

    pub fn reset_for_new_month(&mut self, current_month: &str) {
        if let Some(ref last_month) = self.last_sent_month {
            if last_month != current_month {
                self.completed_this_month = false;
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserReminders {
    pub reminders: HashMap<String, CounterReminder>, // ключ = counter_type.as_str()
    pub global_enabled: bool,
}

impl UserReminders {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            reminders: HashMap::new(),
            global_enabled: true,
        }
    }

    pub fn add_reminder(&mut self, reminder: CounterReminder) {
        self.reminders.insert(reminder.counter_type.as_str().to_string(), reminder);
    }

    pub fn get_reminder_mut(&mut self, counter_type: &CounterType) -> Option<&mut CounterReminder> {
        self.reminders.get_mut(counter_type.as_str())
    }

    pub fn toggle_global(&mut self) -> bool {
        self.global_enabled = !self.global_enabled;
        self.global_enabled
    }
}

#[allow(dead_code)]
pub type RemindersStorage = HashMap<String, UserReminders>; // ключ = chat_id.to_string()

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum ReminderState {
    WaitingForWaterPeriod,
    WaitingForElectricityPeriod,
}

#[allow(dead_code)]
pub type ReminderStates = HashMap<String, ReminderState>; // ключ = chat_id.to_string()