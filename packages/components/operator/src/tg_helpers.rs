use crate::host::{self, LogLevel};
use anyhow::{anyhow, Result};
use tg_utils::telegram::messenger::{
    any_client::TelegramMessengerExt, wasi_client::TelegramMessenger,
};

pub fn read_messages() -> Result<Vec<String>> {
    host::log(LogLevel::Info, "Reading messages...");

    let bot_token = std::env::var("WAVS_ENV_OPERATOR_TELEGRAM_BOT_TOKEN").unwrap_or_default();

    if bot_token.is_empty() {
        return Err(anyhow!(
            "BOT TOKEN is not set in WAVS_ENV_OPERATOR_TELEGRAM_BOT_TOKEN"
        ));
    }

    let tg_messenger = TelegramMessenger::new(bot_token);

    let me = wstd::runtime::block_on(async move { tg_messenger.get_me().await })?;

    println!("{:#?}", me);

    // let messages = requester.read_messages().await?;

    // for message in messages {
    //     println!("Message: {}", message);
    // }

    Ok(vec![])
}
