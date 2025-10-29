mod args;
mod error;
mod handlers;
mod state;

use tg_utils::telegram::{
    api::TelegramUpdate,
    messenger::{any_client::TelegramMessengerExt, reqwest_client::TelegramMessenger},
};
use tg_utils::tracing::tracing_init;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    if dotenvy::dotenv().is_err() {
        tracing::debug!("Failed to load .env file");
    }

    tracing_init();

    let bot_token = std::env::var("SERVER_TELEGRAM_BOT_TOKEN").unwrap_or_default();
    if bot_token.is_empty() {
        panic!("SERVER_TELEGRAM_BOT_TOKEN is not set");
    }
    let messenger = TelegramMessenger::new(bot_token, reqwest::Client::new());

    let mut offset = None;

    loop {
        println!("Waiting for updates");
        // Long poll for 10 seconds, only pick up one message at a time
        let updates = messenger.get_updates(offset, Some(1), Some(10), None).await;
        match updates {
            Ok(updates) => {
                println!("Got {} updates", updates.len());
                for update in updates {
                    println!("Update: {:#?}", update);
                    offset = Some(update.update_id + 1);
                }
            }
            Err(e) => {
                tracing::error!("Failed to get updates: {}", e);
            }
        }
    }
}
