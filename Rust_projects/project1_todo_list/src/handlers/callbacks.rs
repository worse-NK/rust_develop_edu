use teloxide::prelude::*;

use crate::models::{UserState, UserStates};
use crate::storage::StorageType;
use crate::utils::{create_main_menu, create_todo_menu, create_reminder_menu, create_counters_menu};
use crate::models::CounterType;

pub async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    storage: StorageType,
    user_states: UserStates,
) -> ResponseResult<()> {
    if let Some(data) = &q.data {
        let chat_id = q.message.as_ref().unwrap().chat.id;
        
        match data.as_str() {
            "todo_menu" => {
                bot.send_message(chat_id, "📝 TODO List — выберите действие:")
                    .reply_markup(create_todo_menu())
                    .await?;
            }
            "main_menu" => {
                bot.send_message(chat_id, "🏠 Главное меню:")
                    .reply_markup(create_main_menu())
                    .await?;
            }
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
                        .reply_markup(create_todo_menu())
                        .await?;
                } else {
                    bot.send_message(chat_id, "📝 У вас пока нет задач.")
                        .reply_markup(create_todo_menu())
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
                        .reply_markup(create_todo_menu())
                        .await?;
                }
            }
            "remove_task" => {
                let todos = storage.get_tasks(chat_id).await;
                
                if !todos.is_empty() {
                    {
                        let mut states = user_states.lock().await;
                        states.insert(chat_id, UserState::WaitingForRemovalNumber);
                    }
                    
                    bot.send_message(chat_id, "🔢 Введите номер задачи для удаления:")
                        .await?;
                } else {
                    bot.send_message(chat_id, "📝 У вас пока нет задач для удаления.")
                        .reply_markup(create_todo_menu())
                        .await?;
                }
            }
            "clear_all" => {
                if let Err(_) = storage.clear_tasks(chat_id).await {
                    bot.send_message(chat_id, "❌ Ошибка при очистке задач")
                        .reply_markup(create_todo_menu())
                        .await?;
                } else {
                    bot.send_message(chat_id, "🧹 Все задачи очищены")
                        .reply_markup(create_todo_menu())
                        .await?;
                }
            }
            "help" => {
                let help_text = "📖 Справка по боту\n\n\
                    Этот бот помогает вести список задач (TODO List) и напоминать о подаче показаний счетчиков.\n\n\
                    Разделы:\n\
                    • 📝 TODO List — добавляйте задачи по одной или списком, смотрите список, отмечайте выполненными, удаляйте.\n\
                    • ⏰ Напоминалка — настройте период подачи показаний по воде и электричеству, получайте напоминания в нужные дни.\n\n\
                    Используйте кнопки меню для навигации. Данные сохраняются автоматически.";
                
                bot.send_message(chat_id, help_text)
                    .reply_markup(create_main_menu())
                    .await?;
            }
            "todo_help" => {
                let help_text = "📖 Справка по TODO List\n\n\
                    TODO List — это список ваших задач. Можно добавлять задачи по одной или списком,\n\
                    просматривать текущие, отмечать выполненными и удалять.\n\n\
                    Команды:\n\
                    • /add <текст> — добавить задачу\n\
                    • /list — показать все задачи\n\
                    • /done <номер> — отметить выполненной\n\
                    • /remove <номер> — удалить задачу\n\
                    • /clear — очистить все задачи\n\n\
                    Подсказка: удобнее всего пользоваться кнопками меню.";
                
                bot.send_message(chat_id, help_text)
                    .reply_markup(create_todo_menu())
                    .await?;
            }
            "reminder_menu" => {
                bot.send_message(chat_id, "⏰ Напоминалка — выберите действие:")
                    .reply_markup(create_reminder_menu())
                    .await?;
            }
            "reminder_help" => {
                let help_text = "📖 Справка по Напоминалке\n\n\
                    Здесь настраиваются периоды подачи показаний по 💧 воде и ⚡ электричеству.\n\
                    Укажите диапазон дней (например, 16–25). Бот напомнит: в первый день периода, в середине\n\
                    и каждый из последних 3 дней. После подтверждения напоминания прекращаются до следующего месяца.";
                
                bot.send_message(chat_id, help_text)
                    .reply_markup(create_reminder_menu())
                    .await?;
            }
            "counters_menu" => {
                bot.send_message(chat_id, "🏠 Выберите тип счетчика для настройки:")
                    .reply_markup(create_counters_menu())
                    .await?;
            }
            "counter_water" => {
                {
                    let mut states = user_states.lock().await;
                    states.insert(chat_id, UserState::WaitingForWaterPeriod);
                }
                let user = storage.get_user_reminders(chat_id).await;
                let info = user.reminders.get("water").map(|r| format!("Текущий период: {}–{}", r.start_day, r.end_day)).unwrap_or_else(|| "Период не задан".to_string());
                bot.send_message(
                    chat_id,
                    format!(
                        "💧 Настройка напоминаний для счетчика воды\n{}\n\nВведите период в формате: начало-конец (например, 16-25)",
                        info
                    )
                ).await?;
            }
            "counter_electricity" => {
                {
                    let mut states = user_states.lock().await;
                    states.insert(chat_id, UserState::WaitingForElectricityPeriod);
                }
                let user = storage.get_user_reminders(chat_id).await;
                let info = user.reminders.get("electricity").map(|r| format!("Текущий период: {}–{}", r.start_day, r.end_day)).unwrap_or_else(|| "Период не задан".to_string());
                bot.send_message(
                    chat_id,
                    format!(
                        "⚡ Настройка напоминаний для счетчика электричества\n{}\n\nВведите период в формате: начало-конец (например, 16-25)",
                        info
                    )
                ).await?;
            }
            "toggle_reminders" => {
                match storage.toggle_global_reminders(chat_id).await {
                    Ok(enabled) => {
                        let status = if enabled { "включены ✅" } else { "отключены ❌" };
                        bot.send_message(chat_id, format!("🔔 Напоминания {}", status))
                            .reply_markup(create_reminder_menu())
                            .await?;
                    }
                    Err(_) => {
                        bot.send_message(chat_id, "❌ Ошибка при изменении настроек")
                            .reply_markup(create_reminder_menu())
                            .await?;
                    }
                }
            }
            data if data.starts_with("sent_yes_") => {
                let counter_type_str = data.strip_prefix("sent_yes_").unwrap();
                if let Some(counter_type) = CounterType::from_str(counter_type_str) {
                    if let Err(_) = storage.mark_counter_completed(chat_id, counter_type.clone()).await {
                        bot.send_message(chat_id, "❌ Ошибка при сохранении")
                            .await?;
                    } else {
                        bot.send_message(
                            chat_id, 
                            format!("✅ Отлично! Показания {} отмечены как отправленные.\nНапоминания приостановлены до следующего месяца.", counter_type.display_name())
                        ).await?;
                    }
                }
            }
            data if data.starts_with("sent_no_") => {
                let counter_type_str = data.strip_prefix("sent_no_").unwrap();
                if let Some(counter_type) = CounterType::from_str(counter_type_str) {
                    bot.send_message(
                        chat_id, 
                        format!("⏰ Хорошо, я продолжу напоминать о показаниях {}.\nНе забудьте отправить их вовремя!", counter_type.display_name())
                    ).await?;
                }
            }
            _ => {}
        }
        
        bot.answer_callback_query(q.id).await?;
    }
    
    Ok(())
}