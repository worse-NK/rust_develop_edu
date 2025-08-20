use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use chrono::TimeZone;
use teloxide::types::ChatId;
use serde::{Deserialize, Serialize};

use crate::models::{TodoItem, UserReminders, CounterReminder, CounterType};

#[derive(Serialize, Deserialize, Default)]
struct JsonData {
    todos: HashMap<String, Vec<TodoItem>>,
    reminders: HashMap<String, UserReminders>,
}

#[derive(Clone)]
pub struct JsonStorage {
    file_path: String,
    lock: Arc<Mutex<()>>, // синхронизация доступа к файлу
}

impl JsonStorage {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            lock: Arc::new(Mutex::new(())),
        }
    }

    async fn load_data(&self) -> JsonData {
        if !Path::new(&self.file_path).exists() {
            return JsonData::default();
        }

        match fs::read_to_string(&self.file_path).await {
            Ok(content) => {
                serde_json::from_str(&content).unwrap_or_default()
            }
            Err(_) => JsonData::default(),
        }
    }

    async fn save_data(&self, data: &JsonData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::to_string_pretty(data)?;
        let tmp_path = format!("{}.tmp", &self.file_path);
        fs::write(&tmp_path, content).await?;
        fs::rename(&tmp_path, &self.file_path).await?;
        Ok(())
    }

    pub async fn add_task(&self, chat_id: ChatId, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _guard = self.lock.lock().await;
        let mut data = self.load_data().await;
        let chat_key = chat_id.0.to_string();
        
        let todo_item = TodoItem::new(text.to_string());
        let todos = data.todos.entry(chat_key).or_insert_with(Vec::new);
        todos.push(todo_item);
        
        self.save_data(&data).await?;
        Ok(())
    }

    pub async fn get_tasks(&self, chat_id: ChatId) -> Vec<TodoItem> {
        let _guard = self.lock.lock().await;
        let data = self.load_data().await;
        let chat_key = chat_id.0.to_string();
        data.todos.get(&chat_key).cloned().unwrap_or_default()
    }

    pub async fn mark_task_completed(&self, chat_id: ChatId, task_index: usize) -> Result<String, String> {
        let _guard = self.lock.lock().await;
        let mut data = self.load_data().await;
        let chat_key = chat_id.0.to_string();
        
        if let Some(todos) = data.todos.get_mut(&chat_key) {
            if task_index < todos.len() {
                todos[task_index].mark_completed();
                let task_text = todos[task_index].text.clone();
                
                if let Err(_) = self.save_data(&data).await {
                    return Err("Ошибка сохранения".to_string());
                }
                
                Ok(task_text)
            } else {
                Err("Задача с таким номером не найдена".to_string())
            }
        } else {
            Err("У вас нет задач".to_string())
        }
    }

    pub async fn remove_task(&self, chat_id: ChatId, task_index: usize) -> Result<String, String> {
        let _guard = self.lock.lock().await;
        let mut data = self.load_data().await;
        let chat_key = chat_id.0.to_string();
        
        if let Some(todos) = data.todos.get_mut(&chat_key) {
            if task_index < todos.len() {
                let removed_task = todos.remove(task_index);
                
                if let Err(_) = self.save_data(&data).await {
                    return Err("Ошибка сохранения".to_string());
                }
                
                Ok(removed_task.text)
            } else {
                Err("Задача с таким номером не найдена".to_string())
            }
        } else {
            Err("У вас нет задач".to_string())
        }
    }

    pub async fn clear_tasks(&self, chat_id: ChatId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _guard = self.lock.lock().await;
        let mut data = self.load_data().await;
        let chat_key = chat_id.0.to_string();
        
        data.todos.insert(chat_key, Vec::new());
        self.save_data(&data).await?;
        Ok(())
    }

    // Методы для работы с напоминаниями
    pub async fn get_user_reminders(&self, chat_id: ChatId) -> UserReminders {
        let _guard = self.lock.lock().await;
        let data = self.load_data().await;
        let chat_key = chat_id.0.to_string();
        data.reminders.get(&chat_key).cloned().unwrap_or_default()
    }

    pub async fn save_user_reminders(&self, chat_id: ChatId, reminders: UserReminders) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _guard = self.lock.lock().await;
        let mut data = self.load_data().await;
        let chat_key = chat_id.0.to_string();
        data.reminders.insert(chat_key, reminders);
        self.save_data(&data).await
    }

    pub async fn add_counter_reminder(&self, chat_id: ChatId, reminder: CounterReminder) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _guard = self.lock.lock().await;
        let mut data = self.load_data().await;
        let chat_key = chat_id.0.to_string();
        let user_reminders = data.reminders.entry(chat_key).or_insert_with(UserReminders::default);
        user_reminders.add_reminder(reminder);
        self.save_data(&data).await
    }

    pub async fn toggle_global_reminders(&self, chat_id: ChatId) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let _guard = self.lock.lock().await;
        let mut data = self.load_data().await;
        let chat_key = chat_id.0.to_string();
        let user_reminders = data.reminders.entry(chat_key).or_insert_with(UserReminders::default);
        let new_state = user_reminders.toggle_global();
        self.save_data(&data).await?;
        Ok(new_state)
    }

    pub async fn mark_counter_completed(&self, chat_id: ChatId, counter_type: CounterType) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _guard = self.lock.lock().await;
        let mut data = self.load_data().await;
        let chat_key = chat_id.0.to_string();
        let user_reminders = data.reminders.entry(chat_key).or_insert_with(UserReminders::default);
        if let Some(reminder) = user_reminders.get_reminder_mut(&counter_type) {
            reminder.mark_completed();
        }
        self.save_data(&data).await
    }

    pub async fn get_all_reminders(&self) -> HashMap<String, UserReminders> {
        let _guard = self.lock.lock().await;
        let data = self.load_data().await;
        data.reminders
    }

    // Метод для сброса статусов в новом месяце
    pub async fn reset_monthly_statuses(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _guard = self.lock.lock().await;
        let mut data = self.load_data().await;
        let moscow_now = chrono_tz::Europe::Moscow.from_utc_datetime(&chrono::Utc::now().naive_utc());
        let current_month = moscow_now.format("%Y-%m").to_string();
        
        for (_, user_reminders) in data.reminders.iter_mut() {
            for (_, reminder) in user_reminders.reminders.iter_mut() {
                reminder.reset_for_new_month(&current_month);
            }
        }
        
        self.save_data(&data).await
    }}
