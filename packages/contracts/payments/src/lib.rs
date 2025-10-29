#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;
use tg_contract_api::payments::msg::{
    Auth, CustomExecuteMsg, CustomQueryMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use wavs_types::contracts::cosmwasm::service_handler::{
    ServiceHandlerExecuteMessages, ServiceHandlerQueryMessages,
};

use crate::error::ContractError;
use crate::state::{ADMIN, ALLOWED_DENOMS, SERVICE_MANAGER};

mod error;
mod execute;
mod query;
mod state;

#[cfg(test)]
mod tests;

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

    let mut resp = Response::new().add_attribute("method", "instantiate");

    // Set admin or service manager for later validation
    match msg.auth {
        Auth::ServiceManager(service_manager) => {
            let service_manager_addr = deps.api.addr_validate(&service_manager)?;
            SERVICE_MANAGER.save(deps.storage, &service_manager_addr)?;
            resp = resp.add_attribute("service_manager", service_manager);
        }
        Auth::Admin(admin) => {
            let admin_addr = deps.api.addr_validate(&admin)?;
            ADMIN.save(deps.storage, &admin_addr)?;
            resp = resp.add_attribute("admin", admin);
        }
    }

    ALLOWED_DENOMS.save(deps.storage, &msg.allowed_denoms)?;

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Custom(msg) => match msg {
            CustomExecuteMsg::RegisterReceive(msg) => {
                execute::register_receive(deps, env, info, msg)
            }
            CustomExecuteMsg::RegisterSend { tg_handle } => {
                execute::register_send(deps, env, info, tg_handle)
            }
            CustomExecuteMsg::SendPayment(msg) => execute::send_payment(deps, env, info, msg),
        },
        ExecuteMsg::Wavs(msg) => match msg {
            ServiceHandlerExecuteMessages::WavsHandleSignedEnvelope {
                envelope,
                signature_data,
            } => execute::wavs_handle_envelope(deps, env, info, envelope, signature_data),
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Custom(msg) => match msg {
            CustomQueryMsg::AddrByTg { handle } => {
                to_json_binary(&query::addr_by_tg(deps, handle)?)
            }
            CustomQueryMsg::TgByAddr { account } => {
                to_json_binary(&query::tg_by_addr(deps, account)?)
            }
            CustomQueryMsg::Admin {} => to_json_binary(&query::admin(deps)?),
            CustomQueryMsg::PendingPayments { handle } => {
                to_json_binary(&query::pending_payments(deps, handle)?)
            }
            CustomQueryMsg::AllowedDenoms {} => to_json_binary(&query::allowed_denoms(deps)?),
        },
        QueryMsg::Wavs(msg) => match msg {
            ServiceHandlerQueryMessages::WavsServiceManager {} => {
                to_json_binary(&query::wavs_service_manager(deps)?)
            }
        },
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
