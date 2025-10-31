use thiserror::Error;

use crate::telegram::api::bot::TelegramWavsCommandPrefix;

pub type TgResult<T> = Result<T, TelegramBotError>;

#[derive(Debug, Error)]
pub enum TelegramBotError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Message me `/start` to get started")]
    NeedToStart,
    #[error("This command can only be used in direct messages")]
    DirectMessageOnly,
    #[error("Invalid group id")]
    InvalidGroupId,
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    #[error("Invalid command format```Usage:\n{prefix} {format}```", format = prefix.format())]
    InvalidCommandFormat { prefix: TelegramWavsCommandPrefix },
    #[error("Bad command. Try `/help` for more information.")]
    BadCommand,
    #[error("Parse: {0}")]
    Parse(String),
    #[error("This is not a group chat ;)")]
    NotGroupChat,
    #[error("Internal: {0}")]
    Internal(String),
    #[error("Set service: {0:?}")]
    SetService(anyhow::Error),
    #[error("The service has not been set, contact an admin")]
    ServiceNotSet,
    #[error("The service does not have a payments contract, contact an admin")]
    PaymentsContractNotSet,
    #[error("Get service: {0:?}")]
    GetService(anyhow::Error),
    #[error("Error getting status: {0:?}")]
    StatusAny(anyhow::Error),
    #[error("User does not have a username set")]
    NoUsername,
}

impl TelegramBotError {
    pub fn only_respond_to_dm(&self) -> bool {
        match self {
            TelegramBotError::BadCommand => true,
            // for right now let any error go through anywhere else
            _ => false,
        }
    }
}
