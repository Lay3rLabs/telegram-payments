use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint256};

#[cw_serde]
pub struct RegistrationEvent {
    pub tg_handle: String,
    pub address: Addr,
}

impl RegistrationEvent {
    pub const EVENT_TYPE: &'static str = "registration";
    pub const EVENT_ATTR_KEY_TG_HANDLE: &'static str = "tg-handle";
    pub const EVENT_ATTR_KEY_ADDRESS: &'static str = "address";
}

impl From<RegistrationEvent> for cosmwasm_std::Event {
    fn from(src: RegistrationEvent) -> Self {
        cosmwasm_std::Event::new(RegistrationEvent::EVENT_TYPE)
            .add_attribute(RegistrationEvent::EVENT_ATTR_KEY_TG_HANDLE, src.tg_handle)
            .add_attribute(
                RegistrationEvent::EVENT_ATTR_KEY_ADDRESS,
                src.address.to_string(),
            )
    }
}

impl TryFrom<&cosmwasm_std::Event> for RegistrationEvent {
    type Error = anyhow::Error;

    fn try_from(event: &cosmwasm_std::Event) -> Result<Self, Self::Error> {
        if event.ty != Self::EVENT_TYPE && event.ty != format!("wasm-{}", Self::EVENT_TYPE) {
            return Err(anyhow::anyhow!(
                "Expected event type {}, found {}",
                Self::EVENT_TYPE,
                event.ty
            ));
        }

        let mut tg_handle = None;
        let mut address = None;

        for attr in event.attributes.iter() {
            match attr.key.as_str() {
                Self::EVENT_ATTR_KEY_TG_HANDLE => tg_handle = Some(attr.value.to_string()),
                Self::EVENT_ATTR_KEY_ADDRESS => {
                    address = Some(Addr::unchecked(attr.value.to_string()))
                }
                _ => {}
            }
        }

        match (tg_handle, address) {
            (Some(tg_handle), Some(address)) => Ok(Self { tg_handle, address }),
            (Some(_), None) => Err(anyhow::anyhow!(
                "Missing attribute {}",
                Self::EVENT_ATTR_KEY_ADDRESS
            )),
            (None, Some(_)) => Err(anyhow::anyhow!(
                "Missing attribute {}",
                Self::EVENT_ATTR_KEY_TG_HANDLE
            )),
            _ => Err(anyhow::anyhow!("Missing required attributes")),
        }
    }
}

#[cw_serde]
pub struct SendPaymentEvent {
    pub from_tg_handle: String,
    pub to_tg_handle: String,
    pub from_address: Addr,
    pub to_address: Addr,
    pub amount: Uint256,
    pub denom: String,
}

impl SendPaymentEvent {
    pub const EVENT_TYPE: &'static str = "send-payment";
    pub const EVENT_ATTR_KEY_FROM_TG_HANDLE: &'static str = "from-tg-handle";
    pub const EVENT_ATTR_KEY_TO_TG_HANDLE: &'static str = "to-tg-handle";
    pub const EVENT_ATTR_KEY_FROM_ADDRESS: &'static str = "from-address";
    pub const EVENT_ATTR_KEY_TO_ADDRESS: &'static str = "to-address";
    pub const EVENT_ATTR_KEY_AMOUNT: &'static str = "amount";
    pub const EVENT_ATTR_KEY_DENOM: &'static str = "denom";
}

impl From<SendPaymentEvent> for cosmwasm_std::Event {
    fn from(src: SendPaymentEvent) -> Self {
        cosmwasm_std::Event::new(SendPaymentEvent::EVENT_TYPE)
            .add_attribute(
                SendPaymentEvent::EVENT_ATTR_KEY_FROM_TG_HANDLE,
                src.from_tg_handle,
            )
            .add_attribute(
                SendPaymentEvent::EVENT_ATTR_KEY_TO_TG_HANDLE,
                src.to_tg_handle,
            )
            .add_attribute(
                SendPaymentEvent::EVENT_ATTR_KEY_FROM_ADDRESS,
                src.from_address,
            )
            .add_attribute(SendPaymentEvent::EVENT_ATTR_KEY_TO_ADDRESS, src.to_address)
            .add_attribute(SendPaymentEvent::EVENT_ATTR_KEY_AMOUNT, src.amount)
            .add_attribute(SendPaymentEvent::EVENT_ATTR_KEY_DENOM, src.denom)
    }
}

impl TryFrom<&cosmwasm_std::Event> for SendPaymentEvent {
    type Error = anyhow::Error;

    fn try_from(event: &cosmwasm_std::Event) -> Result<Self, Self::Error> {
        if event.ty != Self::EVENT_TYPE && event.ty != format!("wasm-{}", Self::EVENT_TYPE) {
            return Err(anyhow::anyhow!(
                "Expected event type {}, found {}",
                Self::EVENT_TYPE,
                event.ty
            ));
        }

        let mut from_tg_handle = None;
        let mut to_tg_handle = None;
        let mut from_address = None;
        let mut to_address = None;
        let mut amount = None;
        let mut denom = None;

        for attr in event.attributes.iter() {
            match attr.key.as_str() {
                Self::EVENT_ATTR_KEY_FROM_TG_HANDLE => {
                    from_tg_handle = Some(attr.value.to_string())
                }
                Self::EVENT_ATTR_KEY_TO_TG_HANDLE => to_tg_handle = Some(attr.value.to_string()),
                Self::EVENT_ATTR_KEY_FROM_ADDRESS => from_address = Some(attr.value.to_string()),
                Self::EVENT_ATTR_KEY_TO_ADDRESS => to_address = Some(attr.value.to_string()),
                Self::EVENT_ATTR_KEY_AMOUNT => amount = Some(attr.value.to_string()),
                Self::EVENT_ATTR_KEY_DENOM => denom = Some(attr.value.to_string()),
                _ => {}
            }
        }

        let from_tg_handle = match from_tg_handle {
            Some(val) => val,
            None => {
                return Err(anyhow::anyhow!(
                    "Missing attribute {}",
                    Self::EVENT_ATTR_KEY_FROM_TG_HANDLE
                ))
            }
        };

        let to_tg_handle = match to_tg_handle {
            Some(val) => val,
            None => {
                return Err(anyhow::anyhow!(
                    "Missing attribute {}",
                    Self::EVENT_ATTR_KEY_TO_TG_HANDLE
                ))
            }
        };

        let from_address = match from_address {
            Some(val) => Addr::unchecked(val),
            None => {
                return Err(anyhow::anyhow!(
                    "Missing attribute {}",
                    Self::EVENT_ATTR_KEY_FROM_ADDRESS
                ))
            }
        };

        let to_address = match to_address {
            Some(val) => Addr::unchecked(val),
            None => {
                return Err(anyhow::anyhow!(
                    "Missing attribute {}",
                    Self::EVENT_ATTR_KEY_TO_ADDRESS
                ))
            }
        };

        let amount = match amount {
            Some(val) => val.parse::<Uint256>().map_err(|_| {
                anyhow::anyhow!("Invalid attribute {}: {}", Self::EVENT_ATTR_KEY_AMOUNT, val)
            })?,
            None => {
                return Err(anyhow::anyhow!(
                    "Missing attribute {}",
                    Self::EVENT_ATTR_KEY_AMOUNT
                ))
            }
        };

        let denom = match denom {
            Some(val) => val,
            None => {
                return Err(anyhow::anyhow!(
                    "Missing attribute {}",
                    Self::EVENT_ATTR_KEY_DENOM
                ))
            }
        };

        Ok(Self {
            from_tg_handle,
            to_tg_handle,
            from_address,
            to_address,
            amount,
            denom,
        })
    }
}

#[cw_serde]
pub struct ConnectEvent {
    pub tg_handle: String,
    pub address: Addr,
}

impl ConnectEvent {
    pub const EVENT_TYPE: &'static str = "connect";
    pub const EVENT_ATTR_KEY_TG_HANDLE: &'static str = "tg-handle";
    pub const EVENT_ATTR_KEY_ADDRESS: &'static str = "address";
}

impl From<ConnectEvent> for cosmwasm_std::Event {
    fn from(src: ConnectEvent) -> Self {
        cosmwasm_std::Event::new(ConnectEvent::EVENT_TYPE)
            .add_attribute(ConnectEvent::EVENT_ATTR_KEY_TG_HANDLE, src.tg_handle)
            .add_attribute(
                ConnectEvent::EVENT_ATTR_KEY_ADDRESS,
                src.address.to_string(),
            )
    }
}

impl TryFrom<&cosmwasm_std::Event> for ConnectEvent {
    type Error = anyhow::Error;

    fn try_from(event: &cosmwasm_std::Event) -> Result<Self, Self::Error> {
        if event.ty != Self::EVENT_TYPE && event.ty != format!("wasm-{}", Self::EVENT_TYPE) {
            return Err(anyhow::anyhow!(
                "Expected event type {}, found {}",
                Self::EVENT_TYPE,
                event.ty
            ));
        }

        let mut tg_handle = None;
        let mut address = None;

        for attr in event.attributes.iter() {
            match attr.key.as_str() {
                Self::EVENT_ATTR_KEY_TG_HANDLE => tg_handle = Some(attr.value.to_string()),
                Self::EVENT_ATTR_KEY_ADDRESS => {
                    address = Some(Addr::unchecked(attr.value.to_string()))
                }
                _ => {}
            }
        }

        match (tg_handle, address) {
            (Some(tg_handle), Some(address)) => Ok(Self { tg_handle, address }),
            (Some(_), None) => Err(anyhow::anyhow!(
                "Missing attribute {}",
                Self::EVENT_ATTR_KEY_ADDRESS
            )),
            (None, Some(_)) => Err(anyhow::anyhow!(
                "Missing attribute {}",
                Self::EVENT_ATTR_KEY_TG_HANDLE
            )),
            _ => Err(anyhow::anyhow!("Missing required attributes")),
        }
    }
}
