use crate::state::{
    ALLOWED_DENOMS, FUNDED_ACCOUNTS, OPEN_ACCOUNTS, PENDING_PAYMENTS, SERVICE_MANAGER,
};
use cosmwasm_std::{ensure, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::error::ContractError;

pub fn register_receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    tg_handle: String,
    chain_addr: String,
) -> Result<Response, ContractError> {
    // TODO: we probably need a much different way to really handle WAVS, but this is a placeholder
    let service_manager = SERVICE_MANAGER.load(deps.storage)?;
    ensure!(info.sender == service_manager, ContractError::Unauthorized);

    // Don't overwrite anything already registered
    let chain_addr = deps.api.addr_validate(&chain_addr)?;
    if OPEN_ACCOUNTS.has(deps.storage, &tg_handle) {
        return Err(ContractError::TgAlreadyRegistered(tg_handle));
    }
    OPEN_ACCOUNTS.save(deps.storage, &tg_handle, &chain_addr)?;

    // TODO: check if there are any pending payments for this tg_handle and send them

    Ok(Response::new()
        .add_attribute("method", "register_receive")
        .add_attribute("tg_handle", tg_handle)
        .add_attribute("chain_addr", chain_addr))
}

pub fn register_send(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    tg_handle: String,
) -> Result<Response, ContractError> {
    // Don't overwrite anything already registered
    let chain_addr = info.sender;
    if FUNDED_ACCOUNTS.has(deps.storage, &chain_addr) {
        return Err(ContractError::AddrAlreadyRegistered(chain_addr));
    }

    // Ensure this address matches the previous receive registration
    let registered_receive = OPEN_ACCOUNTS.load(deps.storage, &tg_handle)?;
    ensure!(
        registered_receive == chain_addr,
        ContractError::Unauthorized
    ); // TODO: better error message

    FUNDED_ACCOUNTS.save(deps.storage, &chain_addr, &tg_handle)?;

    Ok(Response::new()
        .add_attribute("method", "register_send")
        .add_attribute("tg_handle", tg_handle)
        .add_attribute("chain_addr", chain_addr))
}

pub fn send_payment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from_tg: String,
    to_tg: String,
    amount: Uint128,
    denom: String,
) -> Result<Response, ContractError> {
    // TODO: we probably need a much different way to really handle WAVS, but this is a placeholder
    let service_manager = SERVICE_MANAGER.load(deps.storage)?;
    ensure!(info.sender == service_manager, ContractError::Unauthorized);

    // Check it is an allowed denom
    let allowed_denoms = ALLOWED_DENOMS.load(deps.storage)?;
    ensure!(
        allowed_denoms.contains(&denom),
        ContractError::TokenNotWhitelisted { token: denom }
    );
    // Ensure amount > 0
    ensure!(amount > Uint128::zero(), ContractError::ZeroSend);
    let amount = Coin { amount, denom };

    // Ensure address this account is sending from
    // FIXME: better error messages, not NotFound
    // need to reverse lookup this to ensure it is proper
    let from_addr = OPEN_ACCOUNTS.load(deps.storage, &from_tg)?;
    let check_from = FUNDED_ACCOUNTS.load(deps.storage, &from_addr)?;
    ensure!(check_from == from_tg, ContractError::Unauthorized);

    // Figure out where to send it to
    let to_addr = match OPEN_ACCOUNTS.may_load(deps.storage, &to_tg)? {
        Some(addr) => addr,
        None => {
            // Record the pending payment
            let mut pending = PENDING_PAYMENTS
                .may_load(deps.storage, &to_tg)?
                .unwrap_or_default();
            pending.add_payment(amount.clone());
            PENDING_PAYMENTS.save(deps.storage, &to_tg, &pending)?;

            // Send to this contract
            env.contract.address
        }
    };

    // either send directly to receiver, or we send to self and keep
    let msg = BankMsg::Send {
        // TODO: how do we send from another account? StargateMsg???
        // from_address: from_addr,
        to_address: to_addr.to_string(),
        amount: vec![amount.clone()],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("method", "send_payment")
        .add_attribute("from_tg", from_tg)
        .add_attribute("to_tg", to_tg)
        .add_attribute("amount", amount.amount)
        .add_attribute("denom", amount.denom))
}
