use cosmwasm_std::{CheckedFromRatioError, DecimalRangeExceeded, OverflowError, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
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

    #[error("Must be called by WAVS operators")]
    Unauthorized {},

    #[error("Token not whitelisted: {token}")]
    TokenNotWhitelisted { token: String },

    #[error("Unknown reply id: {id}")]
    UnknownReplyId { id: u64 },
}
