use tg_utils::telegram::api::{TelegramMessage, TelegramUpdate, TelegramWavsCommand};

pub fn parse_update(update: TelegramUpdate) -> Option<(TelegramMessage, TelegramWavsCommand)> {
    let message = update_into_message(update)?;
    let command = message
        .text
        .as_ref()
        .and_then(|text| serde_json::from_str::<TelegramWavsCommand>(&text).ok())?;

    Some((message, command))
}
fn update_into_message(update: TelegramUpdate) -> Option<TelegramMessage> {
    if let Some(message) = update.message {
        Some(message)
    } else if let Some(edited_message) = update.edited_message {
        Some(edited_message)
    } else {
        None
    }
}
