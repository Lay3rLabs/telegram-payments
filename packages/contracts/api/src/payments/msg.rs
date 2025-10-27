use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint256};

#[cw_serde]
pub struct InstantiateMsg {
    pub allowed_denoms: Vec<String>,
    pub service_manager: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ChainAddrResponse)]
    AddrByTg { handle: String },
    #[returns(TgHandleResponse)]
    TgByAddr { account: String },
    #[returns(ServiceManagerResponse)]
    ServiceManager {},
    #[returns(Vec<Coin>)]
    PendingPayments { handle: String },
    #[returns(Vec<String>)]
    AllowedDenoms {},
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Must be called by WAVS operators
    RegisterReceive {
        tg_handle: String,
        chain_addr: String,
    },
    /// Must be called by WAVS operators
    SendPayment {
        from_tg: String,
        to_tg: String,
        amount: Uint256,
        denom: String,
    },
    /// Called directly by the blockchain account authorizing payments
    RegisterSend { tg_handle: String },
}

#[cw_serde]
pub struct TgHandleResponse {
    pub handle: Option<String>,
}

#[cw_serde]
pub struct ChainAddrResponse {
    pub addr: Option<String>,
}

#[cw_serde]
pub struct ServiceManagerResponse {
    pub service_manager: String,
}

#[cw_serde]
pub struct MigrateMsg {}
