use layer_climb::prelude::CosmosAddr;
use tg_utils::{
    client::payments::PaymentsQuerier,
    telegram::{
        api::native::TelegramUser,
        error::{TelegramBotError, TgResult},
    },
};
use tokio::task::spawn_blocking;

use crate::{handlers::tg_webhook::CommandResponse, state::HttpState};

pub async fn query_status(state: HttpState, user: TelegramUser) -> TgResult<CommandResponse> {
    let payments_address = state
        .payments_contract_address()
        .map_err(TelegramBotError::StatusAny)?
        .ok_or(TelegramBotError::PaymentsContractNotSet)?;

    let username = match &user.username {
        Some(name) => name.clone(),
        None => {
            return Err(TelegramBotError::NoUsername);
        }
    };
    // Due to feature unification, we have to assume that the PaymentsQuerier
    // is non-Send, so we spawn a blocking task to run the query.
    let user_address = spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async move {
            let query_client = state
                .get_query_client()
                .await
                .map_err(TelegramBotError::StatusAny)?;

            let payments = PaymentsQuerier::new(query_client.into(), payments_address.into());

            payments
                .addr_by_tg_handle(username)
                .await
                .map_err(TelegramBotError::StatusAny)?
                .map(|addr| CosmosAddr::new_str(&addr, None))
                .transpose()
                .map_err(TelegramBotError::StatusAny)
        })
    })
    .await
    .map_err(|e| TelegramBotError::StatusAny(e.into()))??;

    Ok(CommandResponse::Status {
        address: user_address,
        user,
    })
}
