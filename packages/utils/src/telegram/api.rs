use layer_climb::prelude::CosmosAddr;
use serde::{Deserialize, Serialize};

use crate::telegram::error::TelegramBotError;

#[derive(Clone, Debug)]
pub enum TelegramBotCommand {
    Start,
    Wavs(TelegramWavsCommand),
}

#[derive(Clone, Debug)]
pub enum TelegramWavsCommand {
    Register {
        address: CosmosAddr,
        user_id: i64,
        user_name: Option<String>,
    },
    GroupId {
        group_id: i64,
    },
}

impl TryFrom<&TelegramMessage> for TelegramBotCommand {
    type Error = TelegramBotError;

    fn try_from(message: &TelegramMessage) -> Result<Self, Self::Error> {
        match message.text.clone() {
            Some(text) => {
                let parts: Vec<&str> = text.split_whitespace().collect();

                if parts.len() > 0 {
                    tracing::info!("PARTS: {:?}", parts);
                    match message.chat.chat_type {
                        TelegramChatType::Private => match parts[..] {
                            ["/start"] => Ok(TelegramBotCommand::Start),
                            ["/register", address] => {
                                Ok(TelegramBotCommand::Wavs(TelegramWavsCommand::Register {
                                    user_id: message.from.id,
                                    user_name: message.from.username.clone(),
                                    address: address.parse().map_err(|e| {
                                        TelegramBotError::Parse(format!(
                                            "could not parse {text}: {e:?}"
                                        ))
                                    })?,
                                }))
                            }
                            _ => Err(TelegramBotError::BadCommand(text)),
                        },
                        TelegramChatType::Group
                        | TelegramChatType::SuperGroup
                        | TelegramChatType::Channel => match parts[..] {
                            ["/groupId"] => match message.chat.id {
                                id if id < 0 => {
                                    Ok(TelegramBotCommand::Wavs(TelegramWavsCommand::GroupId {
                                        group_id: id,
                                    }))
                                }
                                _ => Err(TelegramBotError::InvalidGroupId),
                            },
                            _ => Err(TelegramBotError::BadCommand(text)),
                        },
                    }
                } else {
                    Err(TelegramBotError::UnknownCommand(text))
                }
            }
            None => Err(TelegramBotError::EmptyMessage),
        }
    }
}

pub struct TelegramWebHook {}

// https://core.telegram.org/bots/api#update
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramWebHookRequest {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
    pub edited_message: Option<TelegramMessage>,
    pub channel_post: Option<TelegramMessage>,
    pub edited_channel_post: Option<TelegramMessage>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramWebHookResponse {
    pub chat_id: i64,
    pub method: TelegramResponseMethod,
    pub text: String,
}
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum TelegramResponseMethod {
    #[serde(rename = "sendMessage")]
    SendMessge,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub message_thread_id: Option<i64>,
    pub from: TelegramUser,
    pub chat: TelegramChat,
    pub date: u64,
    pub text: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramUser {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub username: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: TelegramChatType,
    pub title: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum TelegramChatType {
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "group")]
    Group,
    #[serde(rename = "supergroup")]
    SuperGroup,
    #[serde(rename = "channel")]
    Channel,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramWebHookInfo {
    pub url: String,
    pub has_custom_certificate: bool,
    pub pending_update_count: u64,
    pub ip_address: Option<String>,
    pub last_error_date: Option<u64>,
    pub last_error_message: Option<String>,
    pub last_synchronization_error_date: Option<u64>,
    pub max_connections: Option<u64>,
    pub allowed_updates: Option<Vec<String>>,
}
