use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use teloxide::types::ChatId;

use crate::models::TodoItem;

pub type TodoStorage = Arc<Mutex<HashMap<ChatId, Vec<TodoItem>>>>;

#[derive(Clone)]
pub struct MemoryStorage {
    storage: TodoStorage,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[allow(dead_code)]
    pub fn get_storage(&self) -> TodoStorage {
        self.storage.clone()
    }

    pub async fn add_task(&self, chat_id: ChatId, text: &str) {
        let todo_item = TodoItem::new(text.to_string());
        let mut storage = self.storage.lock().await;
        let todos = storage.entry(chat_id).or_insert_with(Vec::new);
        todos.push(todo_item);
    }

    pub async fn get_tasks(&self, chat_id: ChatId) -> Vec<TodoItem> {
        let storage = self.storage.lock().await;
        storage.get(&chat_id).cloned().unwrap_or_default()
    }

    pub async fn mark_task_completed(&self, chat_id: ChatId, task_index: usize) -> Result<String, String> {
        let mut storage = self.storage.lock().await;
        let todos = storage.entry(chat_id).or_insert_with(Vec::new);

        if task_index < todos.len() {
            todos[task_index].mark_completed();
            Ok(todos[task_index].text.clone())
        } else {
            Err("Задача с таким номером не найдена".to_string())
        }
    }

    pub async fn remove_task(&self, chat_id: ChatId, task_index: usize) -> Result<String, String> {
        let mut storage = self.storage.lock().await;
        let todos = storage.entry(chat_id).or_insert_with(Vec::new);

        if task_index < todos.len() {
            let removed_task = todos.remove(task_index);
            Ok(removed_task.text)
        } else {
            Err("Задача с таким номером не найдена".to_string())
        }
    }

    pub async fn clear_tasks(&self, chat_id: ChatId) {
        let mut storage = self.storage.lock().await;
        storage.insert(chat_id, Vec::new());
    }
}