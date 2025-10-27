#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;
use tg_contract_api::payments::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::error::ContractError;
use crate::state::{ALLOWED_DENOMS, SERVICE_MANAGER};

mod error;
mod execute;
mod query;
mod state;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Set service manager for later validation
    let service_manager_addr = deps.api.addr_validate(&msg.service_manager)?;
    SERVICE_MANAGER.save(deps.storage, &service_manager_addr)?;

    ALLOWED_DENOMS.save(deps.storage, &msg.allowed_denoms)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("service_manager", msg.service_manager))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterReceive {
            tg_handle,
            chain_addr,
        } => execute::register_receive(deps, env, info, tg_handle, chain_addr),
        ExecuteMsg::RegisterSend { tg_handle } => {
            execute::register_send(deps, env, info, tg_handle)
        }
        ExecuteMsg::SendPayment {
            from_tg,
            to_tg,
            amount,
            denom,
        } => execute::send_payment(deps, env, info, from_tg, to_tg, amount, denom),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AddrByTg { handle } => to_json_binary(&query::addr_by_tg(deps, handle)?),
        QueryMsg::TgByAddr { account } => to_json_binary(&query::tg_by_addr(deps, account)?),
        QueryMsg::ServiceManager {} => to_json_binary(&query::service_manager(deps)?),
        QueryMsg::PendingPayments { handle } => {
            to_json_binary(&query::pending_payments(deps, handle)?)
        }
        QueryMsg::AllowedDenoms {} => to_json_binary(&query::allowed_denoms(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        _ => Err(ContractError::UnknownReplyId { id: msg.id }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
