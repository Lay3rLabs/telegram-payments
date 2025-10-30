use anyhow::{anyhow, Result};
use tg_utils::telegram::{
    api::native::TelegramUpdate,
    messenger::{any_client::TelegramMessengerExt, wasi_client::TelegramMessenger},
};

pub fn get_updates(offset: Option<i64>, limit: Option<u32>) -> Result<Vec<TelegramUpdate>> {
    let bot_token = std::env::var("WAVS_ENV_OPERATOR_TELEGRAM_BOT_TOKEN").unwrap_or_default();

    if bot_token.is_empty() {
        return Err(anyhow!(
            "BOT TOKEN is not set in WAVS_ENV_OPERATOR_TELEGRAM_BOT_TOKEN"
        ));
    }

    let tg_messenger = TelegramMessenger::new(bot_token);

    Ok(wstd::runtime::block_on(async move {
        tg_messenger.get_updates(offset, limit, None, None).await
    })?)
}
