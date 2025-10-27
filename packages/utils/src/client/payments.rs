//! Contract-specific abstraction for different backends (Climb, Climb Pool, MultiTest)
//! Define helper methods here and they'll be available for all backends

use cosmwasm_std::Addr;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::client::{AnyExecutor, AnyQuerier, WavsTxResponse};

#[derive(Clone)]
pub struct PaymentsQuerier {
    pub inner: AnyQuerier,
    pub addr: Addr,
}

impl PaymentsQuerier {
    pub fn new(inner: AnyQuerier, addr: Addr) -> Self {
        Self { inner, addr }
    }
    pub async fn query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &tg_contract_api::payments::msg::QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.inner.contract_query(&self.addr, msg).await
    }
}

#[derive(Clone)]
pub struct PaymentsExecutor {
    pub inner: AnyExecutor,
    pub addr: Addr,
}

impl PaymentsExecutor {
    pub fn new(inner: AnyExecutor, addr: Addr) -> Self {
        Self { inner, addr }
    }
    pub async fn exec(
        &self,
        msg: &tg_contract_api::payments::msg::ExecuteMsg,
        funds: &[cosmwasm_std::Coin],
    ) -> Result<WavsTxResponse, cosmwasm_std::StdError> {
        self.inner.contract_exec(&self.addr, msg, funds).await
    }
}
