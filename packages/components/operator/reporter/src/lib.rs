use tg_components_shared::ReportEvent;
use tg_contract_api::payments::event::{ConnectEvent, RegistrationEvent, SendPaymentEvent};

use crate::{host::LogLevel, wavs::types::events::TriggerData};

// this is needed just to make the ide/compiler happy... we're _always_ compiling to wasm32-wasi
wit_bindgen::generate!({
    world: "wavs-world",
    generate_all,
});

struct Component;

impl Guest for Component {
    fn run(trigger_action: TriggerAction) -> Result<Option<WasmResponse>, String> {
        match trigger_action.data {
            TriggerData::CosmosContractEvent(event_data) => {
                let event = cosmwasm_std::Event::new(event_data.event.ty)
                    .add_attributes(event_data.event.attributes);

                let as_registration =
                    RegistrationEvent::try_from(&event).map(ReportEvent::Registration);
                let as_payment = SendPaymentEvent::try_from(&event).map(ReportEvent::SendPayment);
                let as_connect = ConnectEvent::try_from(&event).map(ReportEvent::Connect);

                let is_registration_ok = as_registration.is_ok();
                let is_payment_ok = as_payment.is_ok();
                let is_connect_ok = as_connect.is_ok();

                match (as_registration, as_payment, as_connect) {
                    (Ok(report), Err(_), Err(_))
                    | (Err(_), Ok(report), Err(_))
                    | (Err(_), Err(_), Ok(report)) => {
                        let wasm_response = WasmResponse {
                            payload: serde_json::to_vec(&report).map_err(|e| e.to_string())?,
                            ordering: None,
                        };
                        Ok(Some(wasm_response))
                    }
                    _ => {
                        if is_registration_ok || is_payment_ok || is_connect_ok {
                            host::log(
                                LogLevel::Error,
                                "Ambiguous event: parsed as multiple event types",
                            );
                        } else {
                            host::log(
                                LogLevel::Warn,
                                "Could not parse event as RegistrationEvent, SendPaymentEvent, or ConnectEvent",
                            );
                        }
                        Ok(None)
                    }
                }
            }
            _ => Ok(None),
        }
    }
}

export!(Component);
