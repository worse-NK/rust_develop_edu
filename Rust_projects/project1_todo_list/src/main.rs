use teloxide::{prelude::*, dptree};

mod config;
mod models;
mod storage;
mod handlers;
mod utils;

use config::Config;
use models::{create_user_states};
use storage::MemoryStorage;
use handlers::commands::Command;

#[tokio::main]
async fn main() {
    // Загружаем переменные из .env файла
    dotenv::dotenv().ok();
    
    pretty_env_logger::init();
    log::info!("Starting Telegram Todo Bot...");

    // Инициализация конфигурации
    let _config = Config::from_env().expect("Failed to load configuration");
    
    let bot = Bot::from_env();
    let storage = MemoryStorage::new();
    let user_states = create_user_states();

    // Создаем обработчики с захваченными зависимостями
    let storage_clone = storage.clone();
    let user_states_clone = user_states.clone();
    let command_handler = move |bot: Bot, msg: Message, cmd: Command| {
        let storage = storage_clone.clone();
        let user_states = user_states_clone.clone();
        async move {
            handlers::handle_command(bot, msg, cmd, storage, user_states).await
        }
    };

    let storage_clone = storage.clone();
    let user_states_clone = user_states.clone();
    let text_handler = move |bot: Bot, msg: Message| {
        let storage = storage_clone.clone();
        let user_states = user_states_clone.clone();
        async move {
            handlers::handle_text_message(bot, msg, storage, user_states).await
        }
    };

    let storage_clone = storage.clone();
    let user_states_clone = user_states.clone();
    let callback_handler = move |bot: Bot, q: CallbackQuery| {
        let storage = storage_clone.clone();
        let user_states = user_states_clone.clone();
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

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}