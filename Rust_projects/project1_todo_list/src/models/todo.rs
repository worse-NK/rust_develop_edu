use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub text: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}

impl TodoItem {
    pub fn new(text: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            text,
            completed: false,
            created_at: Utc::now(),
        }
    }

    pub fn mark_completed(&mut self) {
        self.completed = true;
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }
}