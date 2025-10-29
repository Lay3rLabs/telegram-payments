use crate::{
    host::{self, LogLevel},
    parse::{map_command_to_contract, parse_update},
    tg_helpers::get_updates,
    wavs::types::events::TriggerData,
    TriggerAction, WasmResponse,
};
use anyhow::Result;

// the WasmResponse payload is Vec<ComponentMsg>
pub fn handle_action(trigger_action: TriggerAction) -> Result<Option<WasmResponse>> {
    match trigger_action.data {
        TriggerData::Raw(data) => {
            let data = str::from_utf8(&data)?;
            match data {
                // TODO: include the offset somewhere here for debugging?
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
        TriggerData::Cron(_) => {
            // TODO: we need to store the offset between queries, right?
            let commands = get_updates(None, None)?
                .into_iter()
                .filter_map(parse_update)
                .filter_map(|(_, cmd)| map_command_to_contract(cmd))
                .collect::<Vec<_>>();

            Ok(Some(WasmResponse {
                payload: serde_json::to_vec(&commands)?,
                ordering: None,
            }))
        }
        _ => Ok(None),
    }
}
