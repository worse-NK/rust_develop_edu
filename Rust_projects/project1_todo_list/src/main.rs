use teloxide::{prelude::*, dptree};

mod config;
mod models;
mod storage;
mod handlers;
mod utils;
mod reminder_system;

use config::Config;
use models::{create_user_states};
use storage::JsonStorage;
use handlers::commands::Command;
use reminder_system::ReminderSystem;

#[tokio::main]
async fn main() {
    // Загружаем переменные из .env файла
    dotenv::dotenv().ok();
    
    pretty_env_logger::init();
    log::info!("Starting Telegram Todo Bot...");

    // Инициализация конфигурации
    let _config = Config::from_env().expect("Failed to load configuration");
    
    let bot = Bot::from_env();
    let storage = JsonStorage::new("data/todos.json");
    let user_states = create_user_states();

    // Создаем папку для данных если её нет
    if let Err(_) = tokio::fs::create_dir_all("data").await {
        log::warn!("Could not create data directory");
    }

    // Создаем обработчики с захваченными зависимостями
    // Создаем обработчики с захваченными зависимостями
    let storage_for_commands = storage.clone();
    let user_states_for_commands = user_states.clone();
    let command_handler = move |bot: Bot, msg: Message, cmd: Command| {
        let storage = storage_for_commands.clone();
        let user_states = user_states_for_commands.clone();
        async move {
            handlers::handle_command(bot, msg, cmd, storage, user_states).await
        }
    };

    let storage_for_text = storage.clone();
    let user_states_for_text = user_states.clone();
    let text_handler = move |bot: Bot, msg: Message| {
        let storage = storage_for_text.clone();
        let user_states = user_states_for_text.clone();
        async move {
            handlers::handle_text_message(bot, msg, storage, user_states).await
        }
    };

    let storage_for_callbacks = storage.clone();
    let user_states_for_callbacks = user_states.clone();
    let callback_handler = move |bot: Bot, q: CallbackQuery| {
        let storage = storage_for_callbacks.clone();
        let user_states = user_states_for_callbacks.clone();
        async move {
            handlers::handle_callback(bot, q, storage, user_states).await
        }
    };

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(
                    dptree::entry()
                        .filter_command::<Command>()
                        .endpoint(command_handler)
                )
                .branch(
                    dptree::filter(|msg: Message| msg.text().is_some())
                        .endpoint(text_handler)
                )
        )
        .branch(
            Update::filter_callback_query()
                .endpoint(callback_handler)
        );

    // Запускаем систему напоминаний в отдельной задаче
    let reminder_system = ReminderSystem::new(bot.clone(), storage.clone());
    let reminder_task = tokio::spawn(async move {
        reminder_system.start().await;
    });

    // Запускаем основной диспетчер
    let dispatcher_task = tokio::spawn(async move {
        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    });

    // Ждем завершения любой из задач
    tokio::select! {
        _ = reminder_task => {
            log::info!("Reminder system stopped");
        }
        _ = dispatcher_task => {
            log::info!("Bot dispatcher stopped");
        }
    }
}