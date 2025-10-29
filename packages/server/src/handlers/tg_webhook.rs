use crate::state::HttpState;
use axum::{extract::State, http::Response, response::IntoResponse, Json};
use tg_utils::telegram::{
    api::{
        TelegramBotCommand, TelegramResponseMethod, TelegramWebHookRequest, TelegramWebHookResponse,
    },
    error::{TelegramBotError, TgResult},
};

#[axum::debug_handler]
pub async fn handle_tg_webhook(
    State(state): State<HttpState>,
    Json(req): Json<TelegramWebHookRequest>,
) -> impl IntoResponse {
    tracing::info!("GOT REQUEST: {:?}", req);

    let chat_id = req.message.as_ref().map(|m| m.chat.id);

    let msg = match parse_command(req) {
        Ok(command) => match handle_command(state, command).await {
            Ok(resp_text) => resp_text,
            Err(e) => {
                format!("Error handling command: {:?}", e)
            }
        },
        Err(err) => err.to_string(),
    };

    match chat_id {
        Some(chat_id) => {
            let response = TelegramWebHookResponse {
                method: TelegramResponseMethod::SendMessge,
                chat_id,
                text: msg,
            };

            Json(response).into_response()
        }
        None => Response::new(().into()),
    }
}

fn parse_command(req: TelegramWebHookRequest) -> TgResult<TelegramBotCommand> {
    if let Some(msg) = req.message {
        TelegramBotCommand::try_from(&msg)
    } else {
        Err(TelegramBotError::EmptyMessage)
    }
}

async fn handle_command(state: HttpState, command: TelegramBotCommand) -> TgResult<String> {
    match command.clone() {
        TelegramBotCommand::Start => {
            Ok("Welcome to the bot! Use /register <address> to register.".to_string())
        }
        TelegramBotCommand::Wavs(wavs_command) => match wavs_command.clone() {
            tg_utils::telegram::api::TelegramWavsCommand::Register { address, .. } => {
                state
                    .tg_bot()
                    .send_message_to_group(&format!("{wavs_command:?}"))
                    .await?;
                Ok(format!("okay, you got it, registered {address}"))
                //handle_register(address).await
            }
            tg_utils::telegram::api::TelegramWavsCommand::GroupId { group_id } => {
                Ok(format!("Group ID is {group_id}"))
                //handle_register(address).await
            }
        },
    }
}
