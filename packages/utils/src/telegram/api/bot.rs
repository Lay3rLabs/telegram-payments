use std::str::FromStr;

use crate::telegram::{
    api::native::{TelegramChatType, TelegramMessage},
    error::TelegramBotError,
};
use cosmwasm_std::Uint256;
use layer_climb::prelude::CosmosAddr;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct TelegramBotCommand {
    pub command: TelegramWavsCommand,
    pub raw: TelegramMessage,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TelegramWavsCommand {
    Start,
    Help,
    GroupId {
        group_id: i64,
    },
    Receive {
        address: CosmosAddr,
    },
    Send {
        handle: String,
        amount: Uint256,
        denom: String,
    },
    Admin(TelegramWavsAdminCommand),
    Service,
    Status,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TelegramWavsAdminCommand {
    SetService {
        service_url: String,
        admin_key: String,
    },
}

impl TelegramWavsAdminCommand {
    pub fn admin_key(&self) -> &str {
        match self {
            TelegramWavsAdminCommand::SetService { admin_key, .. } => admin_key,
        }
    }
}

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum TelegramWavsCommandPrefix {
    Start,
    Help,
    GroupId,
    Receive,
    Send,
    Status,
    Admin(TelegramWavsAdminCommandPrefix),
    Service,
}
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum TelegramWavsAdminCommandPrefix {
    SetService,
}

impl TelegramWavsCommandPrefix {
    pub fn format(&self) -> &'static str {
        match self {
            TelegramWavsCommandPrefix::Start => "",
            TelegramWavsCommandPrefix::Help => "",
            TelegramWavsCommandPrefix::GroupId => "",
            TelegramWavsCommandPrefix::Receive => "<address>",
            TelegramWavsCommandPrefix::Send => "<handle> <amount> <denom>",
            TelegramWavsCommandPrefix::Status => "",
            TelegramWavsCommandPrefix::Admin(admin) => match admin {
                TelegramWavsAdminCommandPrefix::SetService => "<service_url> <admin-key>",
            },
            TelegramWavsCommandPrefix::Service => "",
        }
    }
}

impl FromStr for TelegramWavsCommandPrefix {
    type Err = TelegramBotError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "/start" => Ok(TelegramWavsCommandPrefix::Start),
            "/help" => Ok(TelegramWavsCommandPrefix::Help),
            "/groupId" => Ok(TelegramWavsCommandPrefix::GroupId),
            "/receive" => Ok(TelegramWavsCommandPrefix::Receive),
            "/send" => Ok(TelegramWavsCommandPrefix::Send),
            "/status" => Ok(TelegramWavsCommandPrefix::Status),
            "/admin set-service" => Ok(TelegramWavsCommandPrefix::Admin(
                TelegramWavsAdminCommandPrefix::SetService,
            )),
            "/service" => Ok(TelegramWavsCommandPrefix::Service),
            _ => Err(TelegramBotError::UnknownCommand(s.to_string())),
        }
    }
}

impl std::fmt::Display for TelegramWavsCommandPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TelegramWavsCommandPrefix::Start => write!(f, "/start"),
            TelegramWavsCommandPrefix::Help => write!(f, "/help"),
            TelegramWavsCommandPrefix::GroupId => write!(f, "/groupId"),
            TelegramWavsCommandPrefix::Receive => write!(f, "/receive"),
            TelegramWavsCommandPrefix::Send => write!(f, "/send"),
            TelegramWavsCommandPrefix::Status => write!(f, "/status"),
            TelegramWavsCommandPrefix::Admin(TelegramWavsAdminCommandPrefix::SetService) => {
                write!(f, "/admin set-service")
            }
            TelegramWavsCommandPrefix::Service => write!(f, "/service"),
        }
    }
}

impl TryFrom<TelegramMessage> for TelegramBotCommand {
    type Error = TelegramBotError;

    fn try_from(message: TelegramMessage) -> Result<Self, Self::Error> {
        let command = TelegramWavsCommand::try_from(&message)?;
        Ok(TelegramBotCommand {
            command,
            raw: message,
        })
    }
}

impl TryFrom<&TelegramMessage> for TelegramWavsCommand {
    type Error = TelegramBotError;

    fn try_from(message: &TelegramMessage) -> Result<Self, Self::Error> {
        let (prefix, parts) = match message.text.clone() {
            Some(text) => {
                let mut iter = text.split_whitespace();
                let prefix = iter.next().ok_or(TelegramBotError::BadCommand)?;
                let prefix = if prefix == "/admin" {
                    let next = iter.next().ok_or(TelegramBotError::BadCommand)?;
                    TelegramWavsCommandPrefix::from_str(&format!("{} {}", prefix, next))?
                } else {
                    TelegramWavsCommandPrefix::from_str(prefix)?
                };

                (prefix, iter.map(|s| s.to_string()).collect::<Vec<_>>())
            }
            None => {
                return Err(TelegramBotError::BadCommand);
            }
        };

        match prefix {
            TelegramWavsCommandPrefix::Start => Ok(TelegramWavsCommand::Start),
            TelegramWavsCommandPrefix::Help => Ok(TelegramWavsCommand::Help),
            TelegramWavsCommandPrefix::Admin(TelegramWavsAdminCommandPrefix::SetService) => {
                match &parts[..] {
                    [service_url, admin_key] => Ok(TelegramWavsCommand::Admin(
                        TelegramWavsAdminCommand::SetService {
                            service_url: service_url.to_string(),
                            admin_key: admin_key.to_string(),
                        },
                    )),
                    _ => Err(TelegramBotError::InvalidCommandFormat { prefix }),
                }
            }
            TelegramWavsCommandPrefix::Send => match &parts[..] {
                [handle, amount, denom] => Ok(TelegramWavsCommand::Send {
                    handle: handle.to_string(),
                    amount: amount.parse().map_err(|e| {
                        TelegramBotError::Parse(format!("could not parse {amount}: {e:?}"))
                    })?,
                    denom: denom.to_string(),
                }),
                _ => Err(TelegramBotError::InvalidCommandFormat { prefix }),
            },
            TelegramWavsCommandPrefix::Receive => match &parts[..] {
                [address] => Ok(TelegramWavsCommand::Receive {
                    address: address.parse().map_err(|e| {
                        TelegramBotError::Parse(format!("could not parse {address}: {e:?}"))
                    })?,
                }),
                _ => Err(TelegramBotError::InvalidCommandFormat { prefix }),
            },
            TelegramWavsCommandPrefix::Status => Ok(TelegramWavsCommand::Status),
            TelegramWavsCommandPrefix::GroupId => match message.chat.chat_type {
                TelegramChatType::Group
                | TelegramChatType::SuperGroup
                | TelegramChatType::Channel => match message.chat.id {
                    id if id < 0 => Ok(TelegramWavsCommand::GroupId { group_id: id }),
                    _ => Err(TelegramBotError::InvalidGroupId),
                },
                _ => Err(TelegramBotError::NotGroupChat),
            },
            TelegramWavsCommandPrefix::Service => Ok(TelegramWavsCommand::Service),
        }
    }
}
