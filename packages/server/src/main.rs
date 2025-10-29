mod args;
mod error;

use tg_utils::telegram::{
    api::{TelegramBotCommand, TelegramMessage},
    error::TgResult,
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
                    offset = Some(update.update_id + 1);
                    if let Some(msg) = update.message {
                        let chat = msg.chat.id;
                        match TelegramBotCommand::try_from(&msg) {
                            Ok(command) => {
                                handle_command(&messenger, chat, command).await?;
                            }
                            Err(e) => {
                                messenger
                                    .send_message(chat, &format!("Error handling command: {}", e))
                                    .await?;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to get updates: {}", e);
            }
        }
    }
}

async fn handle_command(
    messenger: &TelegramMessenger,
    chat_id: i64,
    command: TelegramBotCommand,
) -> TgResult<TelegramMessage> {
    let msg = match command.clone() {
        TelegramBotCommand::Start => "Welcome to the bot! send /help for help!".to_string(),
        TelegramBotCommand::Wavs(wavs_command) => match wavs_command {
            tg_utils::telegram::api::TelegramWavsCommand::Receive { address, .. } => {
                format!("okay, you got it, registered {address}")
            }
            tg_utils::telegram::api::TelegramWavsCommand::Send { handle, amount, .. } => {
                format!("okay, you got it, sending {amount} to {handle}")
            }
            tg_utils::telegram::api::TelegramWavsCommand::Help {} => {
                r#"*Available commands:*
                    `/start` - Start interaction with the bot
                    `/help` - Show this help message
                    `/status` - Check if your account has been registered for receiving or sending payments
                    `/receive <address>` - Register to receive WAVS payments at the specified address
                    `/send <handle> <amount>` - Register to send WAVS payments to the specified handle
                    "#.to_string()
            }
            tg_utils::telegram::api::TelegramWavsCommand::Status {} => {
                "Checking your current status, please be patient...".to_string()
            }
        },
    };
    messenger.send_message(chat_id, &msg).await
}
