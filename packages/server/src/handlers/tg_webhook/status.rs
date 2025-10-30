use layer_climb::prelude::CosmosAddr;
use tg_utils::{
    client::payments::PaymentsQuerier,
    telegram::{
        api::native::TelegramUser,
        error::{TelegramBotError, TgResult},
    },
};

use crate::{handlers::tg_webhook::CommandResponse, state::HttpState};

pub async fn query_status(state: HttpState, user: TelegramUser) -> TgResult<CommandResponse> {
    let query_client = state
        .get_query_client()
        .await
        .map_err(TelegramBotError::StatusAny)?;

    let payments_address = state
        .payments_contract_address()
        .map_err(TelegramBotError::StatusAny)?
        .ok_or(TelegramBotError::PaymentsContractNotSet)?;

    let payments = PaymentsQuerier::new(query_client.into(), payments_address.into());

    let username = match &user.username {
        Some(name) => name.clone(),
        None => {
            return Err(TelegramBotError::NoUsername);
        }
    };

    let user_address = payments
        .addr_by_tg_handle(username)
        .await
        .map_err(TelegramBotError::StatusAny)?
        .map(|addr| CosmosAddr::new_str(&addr, None))
        .transpose()
        .map_err(TelegramBotError::StatusAny)?;

    Ok(CommandResponse::Status {
        address: user_address,
        user,
    })
}
