use tg_contract_api::payments::msg::ComponentMsg;
use tg_utils::telegram::api::{TelegramMessage, TelegramUpdate, TelegramWavsCommand};

pub fn parse_update(update: TelegramUpdate) -> Option<(TelegramMessage, TelegramWavsCommand)> {
    let message = update_into_message(update)?;
    let command = message
        .text
        .as_ref()
        .and_then(|text| serde_json::from_str::<TelegramWavsCommand>(&text).ok())?;

    Some((message, command))
}

pub fn map_command_to_contract(command: TelegramWavsCommand) -> Option<ComponentMsg> {
    match command {
        TelegramWavsCommand::Receive {
            address,
            user_id,
            user_name,
        } => Some(ComponentMsg::Receive {
            user_id,
            user_name,
            address: address.into(),
        }),
        TelegramWavsCommand::Send {
            handle,
            amount,
            user_id,
            user_name,
        } => Some(ComponentMsg::Send {
            handle,
            amount,
            user_id,
            user_name,
        }),
        _ => None,
    }
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
