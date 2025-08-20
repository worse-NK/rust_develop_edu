use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn create_main_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📝 TODO List", "todo_menu"),
            InlineKeyboardButton::callback("⏰ Напоминалка", "reminder_menu"),
        ],
        vec![
            InlineKeyboardButton::callback("❓ Справка", "help"),
        ],
    ])
}

pub fn create_todo_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("➕ Добавить задачу", "add_task"),
            InlineKeyboardButton::callback("📋 Показать задачи", "list_tasks"),
        ],
        vec![
            InlineKeyboardButton::callback("📝 Добавить список", "add_list"),
            InlineKeyboardButton::callback("✅ Отметить выполненной", "mark_done"),
        ],
        vec![
            InlineKeyboardButton::callback("🗑️ Удалить задачу", "remove_task"),
            InlineKeyboardButton::callback("🧹 Очистить все", "clear_all"),
        ],
        vec![
            InlineKeyboardButton::callback("🔙 Назад в главное меню", "main_menu"),
        ],
    ])
}

pub fn create_reminder_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🏠 Счетчики", "counters_menu"),
        ],
        vec![
            InlineKeyboardButton::callback("🔔 Вкл/Выкл напоминания", "toggle_reminders"),
        ],
        vec![
            InlineKeyboardButton::callback("❓ Справка по напоминаниям", "reminder_help"),
        ],
        vec![
            InlineKeyboardButton::callback("🔙 Назад в главное меню", "main_menu"),
        ],
    ])
}

pub fn create_counters_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("💧 Вода", "counter_water"),
            InlineKeyboardButton::callback("⚡ Электричество", "counter_electricity"),
        ],
        vec![
            InlineKeyboardButton::callback("🔙 Назад к напоминаниям", "reminder_menu"),
        ],
    ])
}

pub fn create_reminder_response_keyboard(counter_type: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("✅ Да, отправил", &format!("sent_yes_{}", counter_type)),
            InlineKeyboardButton::callback("❌ Нет, еще не отправил", &format!("sent_no_{}", counter_type)),
        ],
    ])
}