use crate::state::{
    ALLOWED_DENOMS, FUNDED_ACCOUNTS, OPEN_ACCOUNTS, PENDING_PAYMENTS, SERVICE_MANAGER,
};
use cosmwasm_std::{BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::error::ContractError;

pub fn register_receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    tg_handle: String,
    chain_addr: String,
) -> Result<Response, ContractError> {
    todo!();
}

pub fn register_send(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    tg_handle: String,
) -> Result<Response, ContractError> {
    todo!();
}

pub fn send_payment(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    from_tg: String,
    to_tg: String,
    amount: Uint128,
    denom: String,
) -> Result<Response, ContractError> {
    todo!();
}
