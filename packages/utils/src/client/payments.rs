//! Contract-specific abstraction for different backends (Climb, Climb Pool, MultiTest)
//! Define helper methods here and they'll be available for all backends

use cosmwasm_std::Addr;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::client::{AnyExecutor, AnyQuerier, AnyTxResponse};

use tg_contract_api::payments::msg::{
    AdminResponse, ChainAddrResponse, ExecuteMsg, QueryMsg, RegisterReceiveMsg, TgHandleResponse,
};

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
        msg: &QueryMsg,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        self.inner.contract_query(&self.addr, msg).await
    }

    pub async fn admin(&self) -> Result<Option<String>, cosmwasm_std::StdError> {
        let resp: AdminResponse = self.query(&QueryMsg::Admin {}).await?;

        Ok(resp.admin)
    }

    pub async fn addr_by_tg_handle(
        &self,
        tg_handle: String,
    ) -> Result<Option<String>, cosmwasm_std::StdError> {
        let resp: ChainAddrResponse = self
            .query(&QueryMsg::AddrByTg { handle: tg_handle })
            .await?;

        Ok(resp.addr)
    }

    pub async fn tg_handle_by_addr(
        &self,
        user_addr: String,
    ) -> Result<Option<String>, cosmwasm_std::StdError> {
        let resp: TgHandleResponse = self
            .query(&QueryMsg::TgByAddr { account: user_addr })
            .await?;

        Ok(resp.handle)
    }

    pub async fn allowed_denoms(&self) -> Result<Vec<String>, cosmwasm_std::StdError> {
        self.query(&QueryMsg::AllowedDenoms {}).await
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
        msg: &ExecuteMsg,
        funds: &[cosmwasm_std::Coin],
    ) -> Result<AnyTxResponse, cosmwasm_std::StdError> {
        self.inner.contract_exec(&self.addr, msg, funds).await
    }

    pub async fn register_receive(
        &self,
        tg_handle: String,
        user_addr: Addr,
    ) -> Result<AnyTxResponse, cosmwasm_std::StdError> {
        self.exec(
            &ExecuteMsg::RegisterReceive(RegisterReceiveMsg {
                tg_handle,
                chain_addr: user_addr.to_string(),
            }),
            &[],
        )
        .await
    }
}
