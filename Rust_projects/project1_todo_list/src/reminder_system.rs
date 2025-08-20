use std::time::Duration;
use chrono::{Utc, TimeZone, Timelike, Datelike};
use chrono_tz::Europe::Moscow;
use teloxide::prelude::*;
use tokio::time::interval;

use crate::storage::JsonStorage;
use crate::utils::create_reminder_response_keyboard;

pub struct ReminderSystem {
    bot: Bot,
    storage: JsonStorage,
}

impl ReminderSystem {
    pub fn new(bot: Bot, storage: JsonStorage) -> Self {
        Self { bot, storage }
    }

    pub async fn start(&self) {
        log::info!("Starting reminder system...");
        
        // Запускаем периодическую проверку чаще, чтобы не пропускать окно отправки
        let mut interval = interval(Duration::from_secs(15 * 60)); // каждые 15 минут
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_and_send_reminders().await {
                log::error!("Error in reminder system: {}", e);
            }
        }
    }

    async fn check_and_send_reminders(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        log::info!("Checking reminders...");

        // Сбрасываем статусы для нового месяца
        if let Err(e) = self.storage.reset_monthly_statuses().await {
            log::error!("Failed to reset monthly statuses: {}", e);
        }

        // Получаем текущую дату по московскому времени
        let moscow_now = Moscow.from_utc_datetime(&Utc::now().naive_utc());
        let today = moscow_now.date_naive();
        let current_hour = moscow_now.hour();

        // Отправляем напоминания около 20:00 по Москве, избегая дублей в сутки
        if current_hour < 19 || current_hour > 21 {
            log::debug!("Not reminder window (current hour: {})", current_hour);
            return Ok(());
        }

        // Получаем все настройки напоминаний
        let all_reminders = self.storage.get_all_reminders().await;

        for (chat_id_str, user_reminders) in all_reminders {
            // Пропускаем, если напоминания отключены глобально
            if !user_reminders.global_enabled {
                continue;
            }

            let chat_id = match chat_id_str.parse::<i64>() {
                Ok(id) => ChatId(id),
                Err(_) => continue,
            };

            // Проверяем каждый тип счетчика
            for (_, reminder) in &user_reminders.reminders {
                if reminder.should_remind_today(today) {
                    // проверка на дубли в пределах суток
                    if let Some(last) = &reminder.last_sent_date {
                        if last == &today.format("%Y-%m-%d").to_string() {
                            continue;
                        }
                    }
                    if let Err(e) = self.send_reminder(chat_id, reminder).await {
                        log::error!("Failed to send reminder to {}: {}", chat_id, e);
                    } else {
                        // Отмечаем, что напоминание отправлено в этом месяце
                        if let Err(e) = self.mark_reminder_sent(chat_id, &reminder.counter_type, today).await {
                            log::error!("Failed to mark reminder as sent: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn send_reminder(
        &self,
        chat_id: ChatId,
        reminder: &crate::models::CounterReminder,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let moscow_now = Moscow.from_utc_datetime(&Utc::now().naive_utc());
        let today = moscow_now.date_naive();
        let day = today.day();

        // Определяем тип напоминания
        let reminder_type = if day == reminder.start_day {
            "Начался период подачи показаний"
        } else if day > reminder.end_day.saturating_sub(3) {
            "Скоро заканчивается период подачи показаний"
        } else {
            "Напоминание о подаче показаний"
        };

        let message = format!(
            "⏰ {}\n\n\
            {}\n\
            📅 Период: с {} по {} число\n\
            📊 Сегодня: {} число\n\n\
            Отправили ли вы показания?",
            reminder_type,
            reminder.counter_type.display_name(),
            reminder.start_day,
            reminder.end_day,
            day
        );

        self.bot
            .send_message(chat_id, message)
            .reply_markup(create_reminder_response_keyboard(reminder.counter_type.as_str()))
            .await?;

        log::info!("Sent reminder for {} to {}", reminder.counter_type.as_str(), chat_id);
        Ok(())
    }

    async fn mark_reminder_sent(
        &self,
        chat_id: ChatId,
        counter_type: &crate::models::CounterType,
        date: chrono::NaiveDate,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut user_reminders = self.storage.get_user_reminders(chat_id).await;
        
        if let Some(reminder) = user_reminders.get_reminder_mut(counter_type) {
            reminder.mark_sent(date);
            self.storage.save_user_reminders(chat_id, user_reminders).await?;
        }

        Ok(())
    }

    // Метод для тестирования - отправляет напоминания немедленно
    #[allow(dead_code)]
    pub async fn test_reminders(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        log::info!("Testing reminders (ignoring time check)...");

        let moscow_now = Moscow.from_utc_datetime(&Utc::now().naive_utc());
        let today = moscow_now.date_naive();

        let all_reminders = self.storage.get_all_reminders().await;

        for (chat_id_str, user_reminders) in all_reminders {
            if !user_reminders.global_enabled {
                continue;
            }

            let chat_id = match chat_id_str.parse::<i64>() {
                Ok(id) => ChatId(id),
                Err(_) => continue,
            };

            for (_, reminder) in &user_reminders.reminders {
                if reminder.should_remind_today(today) {
                    self.send_reminder(chat_id, reminder).await?;
                    self.mark_reminder_sent(chat_id, &reminder.counter_type, today).await?;
                }
            }
        }

        Ok(())
    }
}