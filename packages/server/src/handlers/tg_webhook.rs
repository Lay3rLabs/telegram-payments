mod status;

use crate::state::HttpState;
use axum::{extract::State, http::Response, response::IntoResponse, Json};
use cosmwasm_std::Uint256;
use layer_climb::prelude::CosmosAddr;
use status::query_status;
use tg_utils::telegram::{
    api::{
        bot::{
            TelegramBotCommand, TelegramWavsAdminCommand, TelegramWavsAdminCommandPrefix,
            TelegramWavsCommand, TelegramWavsCommandPrefix,
        },
        native::{
            TelegramResponseMethod, TelegramUser, TelegramWebHookRequest, TelegramWebHookResponse,
        },
    },
    error::{TelegramBotError, TgResult},
};

#[cfg(debug_assertions)]
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
    Status {
        address: Option<CosmosAddr>,
        user: TelegramUser,
    },
    Receive {
        address: CosmosAddr,
    },
    Send {
        handle: String,
        amount: Uint256,
        denom: String,
    },
    GroupId {
        group_id: i64,
    },
    SetService {
        service: wavs_types::Service,
    },
    Help,
    Service {
        uri: String,
    },
}

impl std::fmt::Display for CommandResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandResponse::Start { link } => {
                write!(f, "Welcome to the bot!\n\nJoin the group to start receiving and sending WAVS payments.\n\n{link}")
            }
            CommandResponse::Status { address, user } => match address {
                Some(addr) => write!(
                    f,
                    "Hello, {}! Your account is registered with address: {}",
                    user.first_name, addr
                ),
                None => write!(
                    f,
                    "Hello, {}! Your account is not registered yet.",
                    user.first_name
                ),
            },
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

            CommandResponse::SetService { service } => {
                write!(f, "```Service: {:#?}```", service)
            }
            CommandResponse::Service { uri } => {
                write!(f, "Service: {}", uri)
            }
            CommandResponse::Help => {
                let mut s = format!(
                    "*Available commands:*
                `{}` - Start interaction with the bot
                `{}` - Show this help message
                `{}` - Check if your account has been registered for receiving or sending payments
                `{}` - Get the current group chat ID
                `{} {}` - Register to receive WAVS payments at the specified address
                `{} {}` - Register to send WAVS payments to the specified handle
                `{}` - Get the current service information
                `{} {}` - Set the service information (admin only)
                ",
                    TelegramWavsCommandPrefix::Start,
                    TelegramWavsCommandPrefix::Help,
                    TelegramWavsCommandPrefix::Status,
                    TelegramWavsCommandPrefix::GroupId,
                    TelegramWavsCommandPrefix::Receive,
                    TelegramWavsCommandPrefix::Receive.format(),
                    TelegramWavsCommandPrefix::Send,
                    TelegramWavsCommandPrefix::Send.format(),
                    TelegramWavsCommandPrefix::Service,
                    TelegramWavsCommandPrefix::Admin(TelegramWavsAdminCommandPrefix::SetService),
                    TelegramWavsCommandPrefix::Admin(TelegramWavsAdminCommandPrefix::SetService)
                        .format()
                );

                // for every new line, remove whitespace on the next line, but preservie the newline
                s = s
                    .lines()
                    .map(|line| line.trim())
                    .collect::<Vec<&str>>()
                    .join("\n");

                write!(f, "{}", s)
            }
        }
    }
}

async fn handle_command(
    state: HttpState,
    TelegramBotCommand { command, raw }: TelegramBotCommand,
) -> TgResult<CommandResponse> {
    match command.clone() {
        TelegramWavsCommand::Start => {
            let link = state.tg_bot().generate_group_invite_link().await?;
            Ok(CommandResponse::Start { link })
        }
        TelegramWavsCommand::Status {} => query_status(state, raw.from).await,
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
        TelegramWavsCommand::Admin(admin_command) => {
            let expected_admin_key = std::env::var("SERVER_TELEGRAM_ADMIN_KEY").unwrap_or_default();

            if expected_admin_key.is_empty() || admin_command.admin_key() != expected_admin_key {
                return Err(tg_utils::telegram::error::TelegramBotError::Unauthorized.into());
            }

            match admin_command {
                TelegramWavsAdminCommand::SetService {
                    service_url,
                    admin_key: _,
                } => {
                    let service = state
                        .set_service(&service_url)
                        .await
                        .map_err(TelegramBotError::SetService)?;

                    Ok(CommandResponse::SetService { service })
                }
            }
        }
        TelegramWavsCommand::Service => {
            let uri = state
                .get_service_uri()
                .await
                .map_err(TelegramBotError::GetService)?;

            match uri {
                None => Err(TelegramBotError::ServiceNotSet),
                Some(uri) => Ok(CommandResponse::Service { uri }),
            }
        }
    }
}
