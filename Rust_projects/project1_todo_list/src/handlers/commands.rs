use teloxide::{prelude::*, utils::command::BotCommands};

use crate::models::{UserState, UserStates};
use crate::storage::StorageType;
use crate::utils::{create_main_menu, create_todo_menu};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
pub enum Command {
    #[command(description = "начать работу с ботом")]
    Start,
    #[command(description = "показать помощь")]
    Help,
    #[command(description = "добавить задачу")]
    Add(String),
    #[command(description = "показать все задачи")]
    List,
    #[command(description = "отметить задачу как выполненную")]
    Done(String),
    #[command(description = "удалить задачу")]
    Remove(String),
    #[command(description = "очистить все задачи")]
    Clear,
    #[command(description = "тестировать напоминания (только для разработки)")]
    TestReminders,
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    command: Command,
    storage: StorageType,
    user_states: UserStates,
) -> ResponseResult<()> {
    // Сброс состояния пользователя при любой команде
    {
        let mut states = user_states.lock().await;
        states.insert(msg.chat.id, UserState::Default);
    }

    match command {
        Command::Start => {
            let welcome_text = "🤖 Добро пожаловать в Todo Bot!\n\n\
                Я помогу вам управлять списком задач. \
                Выберите действие из меню ниже:";
            
            bot.send_message(msg.chat.id, welcome_text)
                .reply_markup(create_main_menu())
                .await?;
        }
        Command::Help => {
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
            
            bot.send_message(msg.chat.id, help_text)
                .reply_markup(create_main_menu())
                .await?;
        }
        Command::Add(task_text) => {
            if task_text.trim().is_empty() {
                bot.send_message(msg.chat.id, "Пожалуйста, укажите текст задачи. Пример: /add Купить молоко")
                    .reply_markup(create_todo_menu())
                    .await?;
                return Ok(());
            }

            if let Err(_) = storage.add_task(msg.chat.id, task_text.trim()).await {
                bot.send_message(msg.chat.id, "❌ Ошибка при добавлении задачи")
                    .reply_markup(create_todo_menu())
                    .await?;
                return Ok(());
            }
            
            bot.send_message(msg.chat.id, format!("✅ Задача добавлена: {}", task_text.trim()))
                .reply_markup(create_todo_menu())
                .await?;
        }
        Command::List => {
            let todos = storage.get_tasks(msg.chat.id).await;

            if !todos.is_empty() {
                let mut response = "📋 Ваши задачи:\n\n".to_string();
                for (index, todo) in todos.iter().enumerate() {
                    let status = if todo.is_completed() { "✅" } else { "⏳" };
                    response.push_str(&format!("{}. {} {}\n", index + 1, status, todo.text));
                }
                bot.send_message(msg.chat.id, response)
                    .reply_markup(create_todo_menu())
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "📝 У вас пока нет задач.")
                    .reply_markup(create_todo_menu())
                    .await?;
            }
        }
        Command::Done(task_number) => {
            let task_index: usize = match task_number.parse::<usize>() {
                Ok(num) if num > 0 => num - 1,
                _ => {
                    bot.send_message(msg.chat.id, "Пожалуйста, укажите корректный номер задачи. Пример: /done 1")
                        .reply_markup(create_todo_menu())
                        .await?;
                    return Ok(());
                }
            };

            match storage.mark_task_completed(msg.chat.id, task_index).await {
                Ok(task_text) => {
                    bot.send_message(msg.chat.id, format!("✅ Задача \"{}\" отмечена как выполненная!", task_text))
                        .reply_markup(create_todo_menu())
                        .await?;
                }
                Err(error) => {
                    bot.send_message(msg.chat.id, format!("❌ {}", error))
                        .reply_markup(create_todo_menu())
                        .await?;
                }
            }
        }
        Command::Remove(task_number) => {
            let task_index: usize = match task_number.parse::<usize>() {
                Ok(num) if num > 0 => num - 1,
                _ => {
                    bot.send_message(msg.chat.id, "Пожалуйста, укажите корректный номер задачи. Пример: /remove 1")
                        .reply_markup(create_todo_menu())
                        .await?;
                    return Ok(());
                }
            };

            match storage.remove_task(msg.chat.id, task_index).await {
                Ok(task_text) => {
                    bot.send_message(msg.chat.id, format!("🗑️ Задача \"{}\" удалена", task_text))
                        .reply_markup(create_todo_menu())
                        .await?;
                }
                Err(error) => {
                    bot.send_message(msg.chat.id, format!("❌ {}", error))
                        .reply_markup(create_todo_menu())
                        .await?;
                }
            }
        }
        Command::Clear => {
            if let Err(_) = storage.clear_tasks(msg.chat.id).await {
                bot.send_message(msg.chat.id, "❌ Ошибка при очистке задач")
                    .reply_markup(create_todo_menu())
                    .await?;
                return Ok(());
            }
            
            bot.send_message(msg.chat.id, "🧹 Все задачи очищены")
                .reply_markup(create_todo_menu())
                .await?;
        }
        Command::TestReminders => {
            // Эта команда только для тестирования
            bot.send_message(msg.chat.id, "🧪 Команда для тестирования напоминаний доступна только разработчику")
                .reply_markup(create_main_menu())
                .await?;
        }
    }
    Ok(())
}