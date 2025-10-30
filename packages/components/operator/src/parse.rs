use tg_contract_api::payments::msg::ComponentMsg;
use tg_utils::telegram::api::{
    bot::{TelegramBotCommand, TelegramWavsCommand},
    native::{TelegramMessage, TelegramUpdate},
};

pub fn parse_update(update: TelegramUpdate) -> Option<TelegramBotCommand> {
    update_into_message(update).and_then(|text| TelegramBotCommand::try_from(text).ok())
}

pub fn map_command_to_contract(
    TelegramBotCommand { command, raw }: TelegramBotCommand,
) -> Option<ComponentMsg> {
    match command {
        TelegramWavsCommand::Receive { address } => Some(ComponentMsg::Receive {
            address: address.into(),
            user_id: raw.from.id,
            user_name: raw.from.username,
        }),
        TelegramWavsCommand::Send {
            handle,
            amount,
            denom,
        } => Some(ComponentMsg::Send {
            handle,
            amount,
            denom,
            user_id: raw.from.id,
            user_name: raw.from.username,
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
