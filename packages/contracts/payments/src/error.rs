use cosmwasm_std::{Addr, CheckedFromRatioError, DecimalRangeExceeded, OverflowError, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("{0}")]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error("{0}")]
    CheckedFromRatioError(#[from] CheckedFromRatioError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Trying to send 0 tokens")]
    ZeroSend,

    #[error("TG Handle {0} is already registered")]
    TgAlreadyRegistered(String),

    #[error("Address {0} is already registered")]
    AddrAlreadyRegistered(Addr),

    #[error("Token not whitelisted: {token}")]
    TokenNotWhitelisted { token: String },

    #[error("Unknown reply id: {id}")]
    UnknownReplyId { id: u64 },

    #[error("ABI decode: {0}")]
    AbiDecode(String),
}
