use teloxide::prelude::*;

use crate::models::{UserState, UserStates};
use crate::storage::MemoryStorage;
use crate::utils::create_main_menu;

pub async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    storage: MemoryStorage,
    user_states: UserStates,
) -> ResponseResult<()> {
    if let Some(data) = &q.data {
        let chat_id = q.message.as_ref().unwrap().chat.id;
        
        match data.as_str() {
            "add_task" => {
                {
                    let mut states = user_states.lock().await;
                    states.insert(chat_id, UserState::WaitingForTask);
                }
                
                bot.send_message(chat_id, "📝 Введите текст новой задачи:")
                    .await?;
            }
            "add_list" => {
                {
                    let mut states = user_states.lock().await;
                    states.insert(chat_id, UserState::WaitingForTaskList);
                }
                
                bot.send_message(
                    chat_id, 
                    "📝 Введите список задач (каждая с новой строки):\n\n\
                    Примеры:\n\
                    • Купить хлеб\n\
                    • Позвонить маме\n\
                    • Сделать домашку\n\n\
                    Или:\n\
                    1. Первая задача\n\
                    2. Вторая задача\n\
                    3. Третья задача"
                ).await?;
            }
            "list_tasks" => {
                let todos = storage.get_tasks(chat_id).await;

                if !todos.is_empty() {
                    let mut response = "📋 Ваши задачи:\n\n".to_string();
                    for (index, todo) in todos.iter().enumerate() {
                        let status = if todo.is_completed() { "✅" } else { "⏳" };
                        response.push_str(&format!("{}. {} {}\n", index + 1, status, todo.text));
                    }
                    bot.send_message(chat_id, response)
                        .reply_markup(create_main_menu())
                        .await?;
                } else {
                    bot.send_message(chat_id, "📝 У вас пока нет задач.")
                        .reply_markup(create_main_menu())
                        .await?;
                }
            }
            "mark_done" => {
                let todos = storage.get_tasks(chat_id).await;
                
                if !todos.is_empty() {
                    {
                        let mut states = user_states.lock().await;
                        states.insert(chat_id, UserState::WaitingForTaskNumber);
                    }
                    
                    bot.send_message(chat_id, "🔢 Введите номер задачи для отметки как выполненной:")
                        .await?;
                } else {
                    bot.send_message(chat_id, "📝 У вас пока нет задач для отметки.")
                        .reply_markup(create_main_menu())
                        .await?;
                }
            }
            "remove_task" => {
                let todos = storage.get_tasks(chat_id).await;
                
                if !todos.is_empty() {
                    bot.send_message(chat_id, "🔢 Введите номер задачи для удаления:")
                        .await?;
                } else {
                    bot.send_message(chat_id, "📝 У вас пока нет задач для удаления.")
                        .reply_markup(create_main_menu())
                        .await?;
                }
            }
            "clear_all" => {
                storage.clear_tasks(chat_id).await;
                
                bot.send_message(chat_id, "🧹 Все задачи очищены")
                    .reply_markup(create_main_menu())
                    .await?;
            }
            "help" => {
                let help_text = "📖 Справка по использованию бота:\n\n\
                    🔹 Используйте кнопки меню для удобной работы\n\
                    🔹 Или команды:\n\n\
                    /start - главное меню\n\
                    /add <текст> - добавить задачу\n\
                    /list - показать все задачи\n\
                    /done <номер> - отметить выполненной\n\
                    /remove <номер> - удалить задачу\n\
                    /clear - очистить все задачи\n\n\
                    💡 Совет: используйте кнопки - это удобнее!";
                
                bot.send_message(chat_id, help_text)
                    .reply_markup(create_main_menu())
                    .await?;
            }
            _ => {}
        }
        
        bot.answer_callback_query(q.id).await?;
    }
    
    Ok(())
}