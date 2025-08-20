use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use teloxide::types::ChatId;

#[derive(Clone, Debug)]
pub enum UserState {
    Default,
    WaitingForTask,
    WaitingForTaskList,
    WaitingForTaskNumber,
    WaitingForRemovalNumber,
    WaitingForWaterPeriod,
    WaitingForElectricityPeriod,
}

impl Default for UserState {
    fn default() -> Self {
        UserState::Default
    }
}

pub type UserStates = Arc<Mutex<HashMap<ChatId, UserState>>>;

pub fn create_user_states() -> UserStates {
    Arc::new(Mutex::new(HashMap::new()))
}