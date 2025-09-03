pub mod json;
pub mod postgres;

#[allow(dead_code)]
pub mod memory;

pub use json::JsonStorage;
pub use postgres::PostgresStorage;

use std::env;
use teloxide::types::ChatId;
use crate::models::{TodoItem, UserReminders, CounterReminder, CounterType};

#[derive(Clone)]
pub enum StorageType {
    Json(JsonStorage),
    Postgres(PostgresStorage),
}

impl StorageType {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let storage_type = env::var("STORAGE_TYPE").unwrap_or_else(|_| "json".to_string());
        
        match storage_type.as_str() {
            "postgres" => {
                let database_url = env::var("DATABASE_URL")
                    .map_err(|_| "DATABASE_URL not set for PostgreSQL storage")?;
                let postgres_storage = PostgresStorage::new(&database_url).await?;
                Ok(StorageType::Postgres(postgres_storage))
            }
            _ => {
                let json_storage = JsonStorage::new("data/todos.json");
                Ok(StorageType::Json(json_storage))
            }
        }
    }

    pub async fn add_task(&self, chat_id: ChatId, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self {
            StorageType::Json(storage) => storage.add_task(chat_id, text).await,
            StorageType::Postgres(storage) => storage.add_task(chat_id, text).await,
        }
    }

    pub async fn get_tasks(&self, chat_id: ChatId) -> Vec<TodoItem> {
        match self {
            StorageType::Json(storage) => storage.get_tasks(chat_id).await,
            StorageType::Postgres(storage) => storage.get_tasks(chat_id).await,
        }
    }

    pub async fn mark_task_completed(&self, chat_id: ChatId, task_index: usize) -> Result<String, String> {
        match self {
            StorageType::Json(storage) => storage.mark_task_completed(chat_id, task_index).await,
            StorageType::Postgres(storage) => storage.mark_task_completed(chat_id, task_index).await,
        }
    }

    pub async fn remove_task(&self, chat_id: ChatId, task_index: usize) -> Result<String, String> {
        match self {
            StorageType::Json(storage) => storage.remove_task(chat_id, task_index).await,
            StorageType::Postgres(storage) => storage.remove_task(chat_id, task_index).await,
        }
    }

    pub async fn clear_tasks(&self, chat_id: ChatId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self {
            StorageType::Json(storage) => storage.clear_tasks(chat_id).await,
            StorageType::Postgres(storage) => storage.clear_tasks(chat_id).await,
        }
    }

    pub async fn get_user_reminders(&self, chat_id: ChatId) -> UserReminders {
        match self {
            StorageType::Json(storage) => storage.get_user_reminders(chat_id).await,
            StorageType::Postgres(storage) => storage.get_user_reminders(chat_id).await,
        }
    }

    pub async fn save_user_reminders(&self, chat_id: ChatId, reminders: UserReminders) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self {
            StorageType::Json(storage) => storage.save_user_reminders(chat_id, reminders).await,
            StorageType::Postgres(storage) => storage.save_user_reminders(chat_id, reminders).await,
        }
    }

    pub async fn add_counter_reminder(&self, chat_id: ChatId, reminder: CounterReminder) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self {
            StorageType::Json(storage) => storage.add_counter_reminder(chat_id, reminder).await,
            StorageType::Postgres(storage) => storage.add_counter_reminder(chat_id, reminder).await,
        }
    }

    pub async fn toggle_global_reminders(&self, chat_id: ChatId) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            StorageType::Json(storage) => storage.toggle_global_reminders(chat_id).await,
            StorageType::Postgres(storage) => storage.toggle_global_reminders(chat_id).await,
        }
    }

    pub async fn mark_counter_completed(&self, chat_id: ChatId, counter_type: CounterType) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self {
            StorageType::Json(storage) => storage.mark_counter_completed(chat_id, counter_type).await,
            StorageType::Postgres(storage) => storage.mark_counter_completed(chat_id, counter_type).await,
        }
    }

    pub async fn get_all_reminders(&self) -> std::collections::HashMap<String, UserReminders> {
        match self {
            StorageType::Json(storage) => storage.get_all_reminders().await,
            StorageType::Postgres(storage) => storage.get_all_reminders().await,
        }
    }

    pub async fn reset_monthly_statuses(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self {
            StorageType::Json(storage) => storage.reset_monthly_statuses().await,
            StorageType::Postgres(storage) => storage.reset_monthly_statuses().await,
        }
    }
}
