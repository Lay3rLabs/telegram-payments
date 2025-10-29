use crate::{
    host::{self, LogLevel},
    parse::parse_update,
    tg_helpers::get_updates,
    wavs::types::events::TriggerData,
    TriggerAction, WasmResponse,
};
use anyhow::Result;

pub fn handle_action(trigger_action: TriggerAction) -> Result<Option<WasmResponse>> {
    match trigger_action.data {
        TriggerData::Raw(data) => {
            let data = str::from_utf8(&data)?;
            match data {
                "read-updates" => {
                    let updates = get_updates(Some(20i64), None)?;
                    for update in updates {
                        host::log(LogLevel::Info, &format!("Update: {:?}", update));
                    }
                }
                "read-commands" => {
                    let commands = get_updates(None, None)?
                        .into_iter()
                        .filter_map(parse_update)
                        .collect::<Vec<_>>();
                    for command in commands {
                        host::log(LogLevel::Info, &format!("Command: {:?}", command));
                    }
                }
                _ => {
                    return Err(anyhow::anyhow!("Unknown command: {data}"));
                }
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}
