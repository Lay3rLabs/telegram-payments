use serde::{Deserialize, Serialize};
use tg_contract_api::payments::event::{ConnectEvent, RegistrationEvent, SendPaymentEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportEvent {
    Registration(RegistrationEvent),
    SendPayment(SendPaymentEvent),
    Connect(ConnectEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportEventRequest {
    pub event: ReportEvent,
    pub event_id: Vec<u8>,
    pub secret: String,
}
