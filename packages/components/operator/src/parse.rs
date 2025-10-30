use tg_contract_api::payments::msg::{RegisterReceiveMsg, SendPaymentMsg, WavsPayload};
use tg_utils::telegram::api::{
    bot::{TelegramBotCommand, TelegramWavsCommand},
    native::{TelegramMessage, TelegramUpdate},
};

pub fn parse_update(update: TelegramUpdate) -> Option<TelegramBotCommand> {
    update_into_message(update).and_then(|text| TelegramBotCommand::try_from(text).ok())
}

pub fn map_command_to_contract(
    TelegramBotCommand { command, raw }: TelegramBotCommand,
) -> Option<WavsPayload> {
    let from_handle = raw.from.username?;

    match command {
        TelegramWavsCommand::Receive { address } => {
            Some(WavsPayload::Register(RegisterReceiveMsg {
                message_id: raw.message_id,
                chain_addr: address.to_string(),
                tg_handle: from_handle,
            }))
        }
        TelegramWavsCommand::Send {
            handle: to_handle,
            amount,
            denom,
        } => Some(WavsPayload::SendPayment(SendPaymentMsg {
            message_id: raw.message_id,
            from_tg: from_handle,
            to_tg: to_handle,
            amount: amount.into(),
            denom,
        })),
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
