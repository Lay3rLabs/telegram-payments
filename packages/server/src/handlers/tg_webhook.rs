use crate::state::HttpState;
use axum::{extract::State, http::Response, response::IntoResponse, Json};
use layer_climb::prelude::CosmosAddr;
use tg_utils::telegram::{
    api::{
        bot::{TelegramBotCommand, TelegramWavsCommand, TelegramWavsCommandPrefix},
        native::{TelegramResponseMethod, TelegramWebHookRequest, TelegramWebHookResponse},
    },
    error::TgResult,
};

#[axum::debug_handler]
pub async fn handle_tg_webhook(
    State(state): State<HttpState>,
    Json(req): Json<TelegramWebHookRequest>,
) -> impl IntoResponse {
    tracing::info!("GOT REQUEST: {:?}", req);

    let chat_id = req.message.as_ref().map(|m| m.chat.id);

    let message = match req.message {
        Some(msg) => msg,
        None => {
            if req.edited_message.is_some() {
                tracing::debug!("Ignoring edited message");
            } else {
                tracing::debug!("No message found in the request");
            }
            return Response::new(().into());
        }
    };

    if let Some(users) = message.new_chat_members.as_ref().and_then(|users| {
        if users.is_empty() {
            None
        } else {
            Some(users)
        }
    }) {
        for user in users {
            if let Err(e) = state
                .tg_bot()
                .send_message_to_group(&format!(
                    "Welcome, {}!\n\nSend `{}` to see all available commands.\n\nTo get started, send this command to register and receive any funds waiting for you!\n\n```Registration: {} {}```",
                    user.first_name,
                    TelegramWavsCommandPrefix::Help,
                    TelegramWavsCommandPrefix::Receive,
                    TelegramWavsCommandPrefix::Receive.format()
                ))
                .await
            {
                tracing::error!("failed to send welcome message: {e:?}");
            }
        }
    }

    let msg = match TelegramBotCommand::try_from(message) {
        Ok(command) => match handle_command(state, command).await {
            Ok(response) => response.to_string(),
            Err(e) => e.to_string(),
        },
        Err(err) => err.to_string(),
    };

    match (chat_id, !msg.is_empty()) {
        (Some(chat_id), true) => {
            let response = TelegramWebHookResponse {
                method: TelegramResponseMethod::SendMessge,
                chat_id,
                text: msg,
                parse_mode: Some("Markdown".to_string()),
            };

            Json(response).into_response()
        }
        _ => Response::new(().into()),
    }
}

enum CommandResponse {
    Start {
        link: String,
    },
    // TODO - populate the actual data
    Status,
    Receive {
        address: CosmosAddr,
    },
    Send {
        handle: String,
        amount: u64,
        denom: String,
    },
    GroupId {
        group_id: i64,
    },
    Help,
}

impl std::fmt::Display for CommandResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandResponse::Start { link } => {
                write!(f, "Welcome to the bot!\n\nJoin the group to start receiving and sending WAVS payments.\n\n{link}")
            }
            CommandResponse::Status => {
                write!(f, "Checking your current status, please be patient...")
            }
            CommandResponse::Receive { address } => {
                write!(f, "okay, you got it, registered {address}")
            }
            CommandResponse::Send {
                handle,
                amount,
                denom,
            } => {
                write!(f, "okay, you got it, sending {amount} {denom} to {handle}")
            }
            CommandResponse::GroupId { group_id } => {
                write!(f, "Group ID is {group_id}")
            }
            CommandResponse::Help => write!(
                f,
                "*Available commands:*
                `{}` - Start interaction with the bot
                `{}` - Show this help message
                `{}` - Check if your account has been registered for receiving or sending payments
                `{}` - Get the current group chat ID
                `{} {}` - Register to receive WAVS payments at the specified address
                `{} {}` - Register to send WAVS payments to the specified handle
                ",
                TelegramWavsCommandPrefix::Start,
                TelegramWavsCommandPrefix::Help,
                TelegramWavsCommandPrefix::Status,
                TelegramWavsCommandPrefix::GroupId,
                TelegramWavsCommandPrefix::Receive,
                TelegramWavsCommandPrefix::Receive.format(),
                TelegramWavsCommandPrefix::Send,
                TelegramWavsCommandPrefix::Send.format(),
            ),
        }
    }
}

async fn handle_command(
    state: HttpState,
    TelegramBotCommand { command, raw: _ }: TelegramBotCommand,
) -> TgResult<CommandResponse> {
    match command.clone() {
        TelegramWavsCommand::Start => {
            let link = state.tg_bot().generate_group_invite_link().await?;
            Ok(CommandResponse::Start { link })
        }
        TelegramWavsCommand::Status {} => Ok(CommandResponse::Status),
        TelegramWavsCommand::Receive { address, .. } => Ok(CommandResponse::Receive { address }),
        TelegramWavsCommand::Send {
            handle,
            amount,
            denom,
        } => Ok(CommandResponse::Send {
            handle,
            amount,
            denom,
        }),
        TelegramWavsCommand::GroupId { group_id } => Ok(CommandResponse::GroupId { group_id }),
        TelegramWavsCommand::Help {} => Ok(CommandResponse::Help),
    }
}
