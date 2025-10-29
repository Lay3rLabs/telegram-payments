use thiserror::Error;

pub type TgResult<T> = Result<T, TelegramBotError>;

#[derive(Debug, Error)]
pub enum TelegramBotError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Bad command: {0}")]
    BadCommand(String),
    #[error("Invalid group id")]
    InvalidGroupId,
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    #[error("Empty message")]
    EmptyMessage,
    #[error("Parse: {0}")]
    Parse(String),
    #[error("Internal: {0}")]
    Internal(String),
}
