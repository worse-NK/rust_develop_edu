use teloxide::prelude::*;

use crate::models::{UserState, UserStates, CounterType, CounterReminder};
use crate::storage::StorageType;
use crate::utils::{create_main_menu, create_todo_menu, create_reminder_menu, parse_task_list, TaskValidator, TaskIndexValidator, DayValidator, ChatIdValidator, ValidationResult};

pub async fn handle_text_message(
    bot: Bot,
    msg: Message,
    storage: StorageType,
    user_states: UserStates,
) -> ResponseResult<()> {
    let text = match msg.text() {
        Some(text) => text.trim(),
        None => return Ok(()),
    };

    // Валидация Chat ID
    if let ValidationResult::Invalid(error_msg) = ChatIdValidator::validate_chat_id(msg.chat.id.0) {
        log::warn!("Invalid chat ID: {} - {}", msg.chat.id.0, error_msg);
        return Ok(());
    }

    // Создаем валидатор задач
    let task_validator = match TaskValidator::new() {
        Ok(validator) => validator,
        Err(e) => {
            log::error!("Failed to create task validator: {}", e);
            return Ok(());
        }
    };

    // Валидация сообщения
    if let ValidationResult::Invalid(error_msg) = task_validator.validate_message(text) {
        bot.send_message(msg.chat.id, format!("❌ {}", error_msg))
            .reply_markup(create_todo_menu())
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

            // Валидация текста задачи
            match task_validator.validate_task_text(text) {
                ValidationResult::Valid => {
                    // Санитизируем текст перед сохранением
                    let sanitized_text = task_validator.sanitize_task_text(text);
                    
                    if let Err(_) = storage.add_task(msg.chat.id, &sanitized_text).await {
                        bot.send_message(msg.chat.id, "❌ Ошибка при добавлении задачи")
                            .reply_markup(create_todo_menu())
                            .await?;
                        return Ok(());
                    }
                    
                    // Сброс состояния
                    {
                        let mut states = user_states.lock().await;
                        states.insert(msg.chat.id, UserState::Default);
                    }

                    bot.send_message(msg.chat.id, format!("✅ Задача добавлена: {}", sanitized_text))
                        .reply_markup(create_todo_menu())
                        .await?;
                }
                ValidationResult::Invalid(error_msg) => {
                    bot.send_message(msg.chat.id, format!("❌ {}", error_msg))
                        .await?;
                    return Ok(());
                }
            }
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

            let mut added_count = 0;
            let mut valid_tasks = Vec::new();
            
            // Валидируем каждую задачу
            for task in &tasks {
                match task_validator.validate_task_text(task) {
                    ValidationResult::Valid => {
                        let sanitized_task = task_validator.sanitize_task_text(task);
                        if let Ok(_) = storage.add_task(msg.chat.id, &sanitized_task).await {
                            added_count += 1;
                            valid_tasks.push(sanitized_task);
                        }
                    }
                    ValidationResult::Invalid(error_msg) => {
                        log::warn!("Invalid task from user {}: {} - {}", msg.chat.id.0, task, error_msg);
                    }
                }
            }

            // Сброс состояния
            {
                let mut states = user_states.lock().await;
                states.insert(msg.chat.id, UserState::Default);
            }

            if added_count > 0 {
                bot.send_message(
                    msg.chat.id, 
                    format!("✅ Добавлено {} задач:\n{}", 
                        added_count, 
                        valid_tasks.iter().enumerate()
                            .map(|(i, task)| format!("{}. {}", i + 1, task))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                )
                .reply_markup(create_todo_menu())
                .await?;
            } else {
                bot.send_message(msg.chat.id, "❌ Ошибка при добавлении задач или все задачи содержат недопустимые символы")
                    .reply_markup(create_todo_menu())
                    .await?;
            }
        }
        UserState::WaitingForTaskNumber => {
            match text.parse::<usize>() {
                Ok(num) if num > 0 => {
                    let task_index = num - 1;
                    
                    // Получаем список задач для валидации индекса
                    let tasks = storage.get_tasks(msg.chat.id).await;
                    match TaskIndexValidator::validate_task_index(task_index, tasks.len()) {
                        ValidationResult::Valid => {
                            match storage.mark_task_completed(msg.chat.id, task_index).await {
                                Ok(task_text) => {
                                    // Сброс состояния
                                    {
                                        let mut states = user_states.lock().await;
                                        states.insert(msg.chat.id, UserState::Default);
                                    }

                                    bot.send_message(msg.chat.id, format!("✅ Задача \"{}\" отмечена как выполненная!", task_text))
                                        .reply_markup(create_todo_menu())
                                        .await?;
                                }
                                Err(_) => {
                                    bot.send_message(msg.chat.id, "❌ Ошибка при обновлении задачи. Попробуйте еще раз:")
                                        .await?;
                                }
                            }
                        }
                        ValidationResult::Invalid(error_msg) => {
                            bot.send_message(msg.chat.id, format!("❌ {}", error_msg))
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
        UserState::WaitingForRemovalNumber => {
            match text.parse::<usize>() {
                Ok(num) if num > 0 => {
                    let task_index = num - 1;
                    
                    // Получаем список задач для валидации индекса
                    let tasks = storage.get_tasks(msg.chat.id).await;
                    match TaskIndexValidator::validate_task_index(task_index, tasks.len()) {
                        ValidationResult::Valid => {
                            match storage.remove_task(msg.chat.id, task_index).await {
                                Ok(task_text) => {
                                    {
                                        let mut states = user_states.lock().await;
                                        states.insert(msg.chat.id, UserState::Default);
                                    }
                                    
                                    bot.send_message(msg.chat.id, format!("🗑️ Задача \"{}\" удалена", task_text))
                                        .reply_markup(create_todo_menu())
                                        .await?;
                                }
                                Err(_) => {
                                    bot.send_message(msg.chat.id, "❌ Ошибка при удалении задачи. Попробуйте еще раз:")
                                        .await?;
                                }
                            }
                        }
                        ValidationResult::Invalid(error_msg) => {
                            bot.send_message(msg.chat.id, format!("❌ {}", error_msg))
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
        UserState::WaitingForWaterPeriod => {
            handle_period_input(bot, msg.chat.id, storage, user_states, text, CounterType::Water).await?;
        }
        UserState::WaitingForElectricityPeriod => {
            handle_period_input(bot, msg.chat.id, storage, user_states, text, CounterType::Electricity).await?;
        }
        UserState::Default => {
            bot.send_message(msg.chat.id, "🤔 Не понимаю. Используйте кнопки меню или команды.")
                .reply_markup(create_main_menu())
                .await?;
        }
    }

    Ok(())
}

async fn handle_period_input(
    bot: Bot,
    chat_id: ChatId,
    storage: StorageType,
    user_states: UserStates,
    text: &str,
    counter_type: CounterType,
) -> ResponseResult<()> {
    // Парсим период в формате "начало-конец"
    let parts: Vec<&str> = text.split('-').collect();
    if parts.len() != 2 {
        bot.send_message(
            chat_id,
            "❌ Неверный формат. Используйте формат: **начало-конец**\n\
            Например: `16-25` или `1-10`"
        ).await?;
        return Ok(());
    }

    let start_day: u32 = match parts[0].trim().parse() {
        Ok(day) => day,
        _ => {
            bot.send_message(
                chat_id,
                "❌ Неверный формат дня начала. Укажите число от 1 до 31."
            ).await?;
            return Ok(());
        }
    };

    let end_day: u32 = match parts[1].trim().parse() {
        Ok(day) => day,
        _ => {
            bot.send_message(
                chat_id,
                "❌ Неверный формат дня окончания. Укажите число от 1 до 31."
            ).await?;
            return Ok(());
        }
    };

    // Валидация дней с использованием DayValidator
    match DayValidator::validate_day_range(start_day, end_day) {
        ValidationResult::Valid => {
            // Продолжаем обработку
        }
        ValidationResult::Invalid(error_msg) => {
            bot.send_message(chat_id, format!("❌ {}", error_msg)).await?;
            return Ok(());
        }
    }

    // Создаем напоминание
    let reminder = CounterReminder::new(counter_type.clone(), start_day, end_day);
    
    // Сохраняем
    match storage.add_counter_reminder(chat_id, reminder).await {
        Ok(_) => {
            // Сброс состояния
            {
                let mut states = user_states.lock().await;
                states.insert(chat_id, UserState::Default);
            }

            bot.send_message(
                chat_id,
                format!(
                    "✅ Напоминание для {} настроено!\n\n\
                    📅 Период: с {} по {} число каждого месяца\n\
                    🔔 Буду напоминать:\n\
                    • В первый день периода ({})\n\
                    • В середине периода\n\
                    • Каждый день за последние 3 дня\n\n\
                    Напоминания можно отключить в настройках.",
                    counter_type.display_name(),
                    start_day,
                    end_day,
                    start_day
                )
            )
            .reply_markup(create_reminder_menu())
            .await?;
        }
        Err(_) => {
            bot.send_message(chat_id, "❌ Ошибка при сохранении настроек")
                .reply_markup(create_reminder_menu())
                .await?;
        }
    }

    Ok(())
}