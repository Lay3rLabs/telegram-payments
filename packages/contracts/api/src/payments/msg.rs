use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint256;
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
    pub message_id: i64,
    pub tg_handle: String,
    pub chain_addr: String,
}

#[cw_serde]
pub struct SendPaymentMsg {
    pub message_id: i64,
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

impl WavsPayload {
    pub fn message_id(&self) -> i64 {
        match self {
            WavsPayload::Register(msg) => msg.message_id,
            WavsPayload::SendPayment(msg) => msg.message_id,
        }
    }

    pub fn encode(&self) -> cosmwasm_std::StdResult<Vec<u8>> {
        cosmwasm_std::to_json_vec(self)
    }

    pub fn decode(bytes: impl AsRef<[u8]>) -> cosmwasm_std::StdResult<Self> {
        cosmwasm_std::from_json(bytes)
    }
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
