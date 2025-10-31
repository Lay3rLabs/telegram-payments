use serde::{Deserialize, Serialize};
use tg_contract_api::payments::event::{RegistrationEvent, SendPaymentEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportEvent {
    Registration(RegistrationEvent),
    SendPayment(SendPaymentEvent),
}
