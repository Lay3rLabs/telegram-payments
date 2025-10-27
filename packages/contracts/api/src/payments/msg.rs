use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint256};
use wavs_types::contracts::cosmwasm::service_handler::{
    ServiceHandlerExecuteMessages, ServiceHandlerQueryMessages,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub allowed_denoms: Vec<String>,
    pub auth: Auth,
}

#[cw_serde]
pub enum Auth {
    /// Implement ServiceHandler interface, validate signatures with the ServiceManager
    ServiceManager(String),
    /// Used for tests. One account is authorized to execute the privileged methods normally reserved for WAVS
    Admin(String),
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
    #[serde(untagged)]
    #[returns(())]
    Wavs(ServiceHandlerQueryMessages),
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
    #[serde(untagged)]
    Wavs(ServiceHandlerExecuteMessages),
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
