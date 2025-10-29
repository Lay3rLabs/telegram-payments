use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint256};
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
#[schemaifier(mute_warnings)]
#[derive(QueryResponses)]
#[query_responses(nested)]
#[serde(untagged)]
pub enum QueryMsg {
    Custom(CustomQueryMsg),
    Wavs(ServiceHandlerQueryMessages),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum CustomQueryMsg {
    #[returns(ChainAddrResponse)]
    AddrByTg { handle: String },
    #[returns(TgHandleResponse)]
    TgByAddr { account: String },
    #[returns(AdminResponse)]
    Admin {},
    #[returns(Vec<cosmwasm_std::Coin>)]
    PendingPayments { handle: String },
    #[returns(Vec<String>)]
    AllowedDenoms {},
}

#[cw_serde]
pub enum ComponentMsg {
    Receive {
        address: Addr,
        user_id: i64,
        user_name: Option<String>,
    },
    Send {
        handle: String,
        amount: u64,
        user_id: i64,
        user_name: Option<String>,
    },
}

#[cw_serde]
#[schemaifier(mute_warnings)]
#[serde(untagged)]
pub enum ExecuteMsg {
    Custom(CustomExecuteMsg),
    Wavs(ServiceHandlerExecuteMessages),
}

#[cw_serde]
pub enum CustomExecuteMsg {
    /// Must be called by WAVS operators
    RegisterReceive(RegisterReceiveMsg),
    /// Must be called by WAVS operators
    SendPayment(SendPaymentMsg),
    /// Called directly by the blockchain account authorizing payments
    RegisterSend { tg_handle: String },
}

#[cw_serde]
pub struct RegisterReceiveMsg {
    pub tg_handle: String,
    pub chain_addr: String,
}

#[cw_serde]
pub struct SendPaymentMsg {
    pub from_tg: String,
    pub to_tg: String,
    pub amount: Uint256,
    pub denom: String,
}

#[cw_serde]
pub enum WavsPayload {
    Register(RegisterReceiveMsg),
    SendPayment(SendPaymentMsg),
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
pub struct AdminResponse {
    pub admin: Option<String>,
}

#[cw_serde]
pub struct MigrateMsg {}
