use tg_components_shared::ReportEvent;
use tg_contract_api::payments::event::{RegistrationEvent, SendPaymentEvent};

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

                match (as_registration, as_payment) {
                    (Ok(report), Err(_)) | (Err(_), Ok(report)) => {
                        let wasm_response = WasmResponse {
                            payload: serde_json::to_vec(&report).map_err(|e| e.to_string())?,
                            ordering: None,
                        };
                        Ok(Some(wasm_response))
                    }
                    (Ok(_), Ok(_)) => {
                        host::log(
                            LogLevel::Error,
                            "Ambiguous event: parsed as both RegistrationEvent and SendPaymentEvent",
                        );
                        Ok(None)
                    }
                    _ => {
                        host::log(
                            LogLevel::Warn,
                            "Could not parse event as either RegistrationEvent or SendPaymentEvent",
                        );
                        Ok(None)
                    }
                }
            }
            _ => Ok(None),
        }
    }
}

export!(Component);
