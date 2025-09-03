use sqlx::{PgPool, Row};
use teloxide::types::ChatId;
use chrono::TimeZone;

use crate::models::{TodoItem, UserReminders, CounterReminder, CounterType};

#[derive(Clone)]
pub struct PostgresStorage {
    pool: PgPool,
}

impl PostgresStorage {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;
        
        // Создаем таблицы если их нет
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS todos (
                id VARCHAR PRIMARY KEY,
                chat_id BIGINT NOT NULL,
                text TEXT NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#
        ).execute(&pool).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_reminders (
                chat_id BIGINT PRIMARY KEY,
                global_enabled BOOLEAN NOT NULL DEFAULT TRUE
            )
            "#
        ).execute(&pool).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS counter_reminders (
                chat_id BIGINT NOT NULL,
                counter_type VARCHAR NOT NULL,
                start_day INTEGER NOT NULL,
                end_day INTEGER NOT NULL,
                enabled BOOLEAN NOT NULL DEFAULT TRUE,
                last_sent_month VARCHAR,
                last_sent_date VARCHAR,
                completed_this_month BOOLEAN NOT NULL DEFAULT FALSE,
                PRIMARY KEY (chat_id, counter_type)
            )
            "#
        ).execute(&pool).await?;

        Ok(Self { pool })
    }

    // Методы для работы с задачами
    pub async fn add_task(&self, chat_id: ChatId, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let todo_item = TodoItem::new(text.to_string());
        
        sqlx::query(
            "INSERT INTO todos (id, chat_id, text, completed, created_at) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(&todo_item.id)
        .bind(chat_id.0)
        .bind(&todo_item.text)
        .bind(todo_item.completed)
        .bind(todo_item.created_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_tasks(&self, chat_id: ChatId) -> Vec<TodoItem> {
        let rows = sqlx::query(
            "SELECT id, text, completed, created_at FROM todos WHERE chat_id = $1 ORDER BY created_at"
        )
        .bind(chat_id.0)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        rows.into_iter().map(|row| TodoItem {
            id: row.get("id"),
            text: row.get("text"),
            completed: row.get("completed"),
            created_at: row.get("created_at"),
        }).collect()
    }

    pub async fn mark_task_completed(&self, chat_id: ChatId, task_index: usize) -> Result<String, String> {
        let tasks = self.get_tasks(chat_id).await;
        
        if task_index >= tasks.len() {
            return Err("Задача с таким номером не найдена".to_string());
        }

        let task = &tasks[task_index];
        
        sqlx::query("UPDATE todos SET completed = TRUE WHERE id = $1")
            .bind(&task.id)
            .execute(&self.pool)
            .await
            .map_err(|_| "Ошибка сохранения".to_string())?;

        Ok(task.text.clone())
    }

    pub async fn remove_task(&self, chat_id: ChatId, task_index: usize) -> Result<String, String> {
        let tasks = self.get_tasks(chat_id).await;
        
        if task_index >= tasks.len() {
            return Err("Задача с таким номером не найдена".to_string());
        }

        let task = &tasks[task_index];
        let task_text = task.text.clone();
        
        sqlx::query("DELETE FROM todos WHERE id = $1")
            .bind(&task.id)
            .execute(&self.pool)
            .await
            .map_err(|_| "Ошибка сохранения".to_string())?;

        Ok(task_text)
    }

    pub async fn clear_tasks(&self, chat_id: ChatId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query("DELETE FROM todos WHERE chat_id = $1")
            .bind(chat_id.0)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    // Методы для работы с напоминаниями
    pub async fn get_user_reminders(&self, chat_id: ChatId) -> UserReminders {
        let user_row = sqlx::query(
            "SELECT global_enabled FROM user_reminders WHERE chat_id = $1"
        )
        .bind(chat_id.0)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None);

        let global_enabled = user_row.map(|row| row.get("global_enabled")).unwrap_or(true);

        let counter_rows = sqlx::query(
            "SELECT counter_type, start_day, end_day, enabled, last_sent_month, last_sent_date, completed_this_month 
             FROM counter_reminders WHERE chat_id = $1"
        )
        .bind(chat_id.0)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let mut reminders = std::collections::HashMap::new();
        for row in counter_rows {
            let counter_type_str: String = row.get("counter_type");
            let counter_type = CounterType::from_str(&counter_type_str).unwrap_or(CounterType::Water);
            
            let reminder = CounterReminder {
                counter_type: counter_type.clone(),
                start_day: row.get::<i32, _>("start_day") as u32,
                end_day: row.get::<i32, _>("end_day") as u32,
                enabled: row.get("enabled"),
                last_sent_month: row.get("last_sent_month"),
                last_sent_date: row.get("last_sent_date"),
                completed_this_month: row.get("completed_this_month"),
            };
            
            reminders.insert(counter_type.as_str().to_string(), reminder);
        }

        UserReminders {
            reminders,
            global_enabled,
        }
    }

    pub async fn save_user_reminders(&self, chat_id: ChatId, reminders: UserReminders) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Сохраняем глобальные настройки
        sqlx::query(
            "INSERT INTO user_reminders (chat_id, global_enabled) VALUES ($1, $2) 
             ON CONFLICT (chat_id) DO UPDATE SET global_enabled = $2"
        )
        .bind(chat_id.0)
        .bind(reminders.global_enabled)
        .execute(&self.pool)
        .await?;

        // Сохраняем напоминания по счетчикам
        for (_, reminder) in reminders.reminders {
            sqlx::query(
                "INSERT INTO counter_reminders (chat_id, counter_type, start_day, end_day, enabled, last_sent_month, last_sent_date, completed_this_month) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                 ON CONFLICT (chat_id, counter_type) DO UPDATE SET 
                 start_day = $3, end_day = $4, enabled = $5, last_sent_month = $6, last_sent_date = $7, completed_this_month = $8"
            )
            .bind(chat_id.0)
            .bind(reminder.counter_type.as_str())
            .bind(reminder.start_day as i32)
            .bind(reminder.end_day as i32)
            .bind(reminder.enabled)
            .bind(&reminder.last_sent_month)
            .bind(&reminder.last_sent_date)
            .bind(reminder.completed_this_month)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn add_counter_reminder(&self, chat_id: ChatId, reminder: CounterReminder) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut user_reminders = self.get_user_reminders(chat_id).await;
        user_reminders.add_reminder(reminder);
        self.save_user_reminders(chat_id, user_reminders).await
    }

    pub async fn toggle_global_reminders(&self, chat_id: ChatId) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut user_reminders = self.get_user_reminders(chat_id).await;
        let new_state = user_reminders.toggle_global();
        self.save_user_reminders(chat_id, user_reminders).await?;
        Ok(new_state)
    }

    pub async fn mark_counter_completed(&self, chat_id: ChatId, counter_type: CounterType) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            "UPDATE counter_reminders SET completed_this_month = TRUE WHERE chat_id = $1 AND counter_type = $2"
        )
        .bind(chat_id.0)
        .bind(counter_type.as_str())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_all_reminders(&self) -> std::collections::HashMap<String, UserReminders> {
        let mut result = std::collections::HashMap::new();
        
        // Получаем всех пользователей с напоминаниями
        let chat_ids: Vec<i64> = sqlx::query("SELECT DISTINCT chat_id FROM user_reminders")
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|row| row.get("chat_id"))
            .collect();

        for chat_id in chat_ids {
            let reminders = self.get_user_reminders(ChatId(chat_id)).await;
            result.insert(chat_id.to_string(), reminders);
        }

        result
    }

    pub async fn reset_monthly_statuses(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let moscow_now = chrono_tz::Europe::Moscow.from_utc_datetime(&chrono::Utc::now().naive_utc());
        let current_month = moscow_now.format("%Y-%m").to_string();
        
        // Сбрасываем статусы для всех пользователей
        sqlx::query(
            "UPDATE counter_reminders SET completed_this_month = FALSE 
             WHERE last_sent_month IS NULL OR last_sent_month != $1"
        )
        .bind(&current_month)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
