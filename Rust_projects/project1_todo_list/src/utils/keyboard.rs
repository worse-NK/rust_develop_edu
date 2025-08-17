use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn create_main_menu() -> InlineKeyboardMarkup {
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
            InlineKeyboardButton::callback("❓ Справка", "help"),
        ],
    ])
}