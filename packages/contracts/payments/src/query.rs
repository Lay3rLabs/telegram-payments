use crate::state::{
    ALLOWED_DENOMS, FUNDED_ACCOUNTS, OPEN_ACCOUNTS, PENDING_PAYMENTS, SERVICE_MANAGER,
};
use cosmwasm_std::{Coin, Deps, StdResult};
use tg_contract_api::payments::msg::{ChainAddrResponse, ServiceManagerResponse, TgHandleResponse};

pub fn addr_by_tg(deps: Deps, handle: String) -> StdResult<ChainAddrResponse> {
    let addr = OPEN_ACCOUNTS
        .may_load(deps.storage, &handle)?
        .map(Into::into);
    Ok(ChainAddrResponse { addr })
}

pub fn tg_by_addr(deps: Deps, account: String) -> StdResult<TgHandleResponse> {
    let addr = deps.api.addr_validate(&account)?;
    let handle = FUNDED_ACCOUNTS.may_load(deps.storage, &addr)?;
    Ok(TgHandleResponse { handle })
}

pub fn allowed_denoms(deps: Deps) -> StdResult<Vec<String>> {
    ALLOWED_DENOMS.load(deps.storage)
}

pub fn service_manager(deps: Deps) -> StdResult<ServiceManagerResponse> {
    let service_manager = SERVICE_MANAGER.load(deps.storage)?.into();
    Ok(ServiceManagerResponse { service_manager })
}

pub fn pending_payments(deps: Deps, handle: String) -> StdResult<Vec<Coin>> {
    let loaded = PENDING_PAYMENTS.may_load(deps.storage, &handle)?;
    let payments = loaded.map(|p| p.balance()).unwrap_or_default();
    Ok(payments)
}
