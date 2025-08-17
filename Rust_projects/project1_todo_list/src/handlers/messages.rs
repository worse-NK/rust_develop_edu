use teloxide::prelude::*;

use crate::models::{UserState, UserStates};
use crate::storage::MemoryStorage;
use crate::utils::{create_main_menu, parse_task_list};

pub async fn handle_text_message(
    bot: Bot,
    msg: Message,
    storage: MemoryStorage,
    user_states: UserStates,
) -> ResponseResult<()> {
    let text = match msg.text() {
        Some(text) => text.trim(),
        None => return Ok(()),
    };

    // Проверка на безопасность - ограничение длины сообщения
    if text.len() > 4000 {
        bot.send_message(msg.chat.id, "❌ Сообщение слишком длинное. Максимум 4000 символов.")
            .reply_markup(create_main_menu())
            .await?;
        return Ok(());
    }

    let current_state = {
        let states = user_states.lock().await;
        states.get(&msg.chat.id).cloned().unwrap_or_default()
    };

    match current_state {
        UserState::WaitingForTask => {
            if text.is_empty() {
                bot.send_message(msg.chat.id, "❌ Пожалуйста, введите текст задачи:")
                    .await?;
                return Ok(());
            }

            storage.add_task(msg.chat.id, text).await;
            
            // Сброс состояния
            {
                let mut states = user_states.lock().await;
                states.insert(msg.chat.id, UserState::Default);
            }

            bot.send_message(msg.chat.id, format!("✅ Задача добавлена: {}", text))
                .reply_markup(create_main_menu())
                .await?;
        }
        UserState::WaitingForTaskList => {
            if text.is_empty() {
                bot.send_message(msg.chat.id, "❌ Пожалуйста, введите список задач:")
                    .await?;
                return Ok(());
            }

            let tasks = parse_task_list(text);
            if tasks.is_empty() {
                bot.send_message(msg.chat.id, "❌ Не удалось распознать задачи. Попробуйте еще раз:")
                    .await?;
                return Ok(());
            }

            for task in &tasks {
                storage.add_task(msg.chat.id, task).await;
            }

            // Сброс состояния
            {
                let mut states = user_states.lock().await;
                states.insert(msg.chat.id, UserState::Default);
            }

            bot.send_message(
                msg.chat.id, 
                format!("✅ Добавлено {} задач:\n{}", 
                    tasks.len(), 
                    tasks.iter().enumerate()
                        .map(|(i, task)| format!("{}. {}", i + 1, task))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            )
            .reply_markup(create_main_menu())
            .await?;
        }
        UserState::WaitingForTaskNumber => {
            match text.parse::<usize>() {
                Ok(num) if num > 0 => {
                    let task_index = num - 1;
                    
                    match storage.mark_task_completed(msg.chat.id, task_index).await {
                        Ok(task_text) => {
                            // Сброс состояния
                            {
                                let mut states = user_states.lock().await;
                                states.insert(msg.chat.id, UserState::Default);
                            }

                            bot.send_message(msg.chat.id, format!("✅ Задача \"{}\" отмечена как выполненная!", task_text))
                                .reply_markup(create_main_menu())
                                .await?;
                        }
                        Err(_) => {
                            bot.send_message(msg.chat.id, "❌ Задача с таким номером не найдена. Попробуйте еще раз:")
                                .await?;
                        }
                    }
                }
                _ => {
                    bot.send_message(msg.chat.id, "❌ Пожалуйста, введите корректный номер задачи:")
                        .await?;
                }
            }
        }
        UserState::Default => {
            bot.send_message(msg.chat.id, "🤔 Не понимаю. Используйте кнопки меню или команды.")
                .reply_markup(create_main_menu())
                .await?;
        }
    }

    Ok(())
}