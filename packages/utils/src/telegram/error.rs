use thiserror::Error;

use crate::telegram::api::bot::TelegramWavsCommandPrefix;

pub type TgResult<T> = Result<T, TelegramBotError>;

#[derive(Debug, Error)]
pub enum TelegramBotError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Invalid group id")]
    InvalidGroupId,
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    #[error("Invalid command format```Usage:\n{prefix} {format}```", format = prefix.format())]
    InvalidCommandFormat { prefix: TelegramWavsCommandPrefix },
    #[error("")]
    EmptyMessage,
    #[error("Parse: {0}")]
    Parse(String),
    #[error("This is not a group chat ;)")]
    NotGroupChat,
    #[error("Internal: {0}")]
    Internal(String),
}
