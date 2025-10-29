use crate::{
    host::{self, LogLevel},
    tg_helpers::read_messages,
    wavs::types::events::TriggerData,
    TriggerAction, WasmResponse,
};
use anyhow::Result;

pub fn handle_action(trigger_action: TriggerAction) -> Result<Option<WasmResponse>> {
    match trigger_action.data {
        TriggerData::Raw(data) => {
            let data = str::from_utf8(&data)?;
            match data {
                "read-messages" => {
                    let messages = read_messages()?;
                    for message in messages {
                        host::log(LogLevel::Info, &format!("Message: {}", message,));
                    }
                }
                _ => {
                    host::log(LogLevel::Warn, &format!("Unknown command: {data}",));
                }
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}
