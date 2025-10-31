use crate::{
    host::{self, LogLevel},
    parse::{map_command_to_contract, parse_update},
    state::{acquire_lock, get_offset, release_lock, set_offset},
    tg_helpers::get_updates,
    wavs::types::events::TriggerData,
    TriggerAction, WasmResponse,
};
use anyhow::Result;
use tg_contract_api::payments::msg::WavsPayload;

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
                        println!("Command: {:?}", command);
                    }
                }
                "read-real" => {
                    let commands = get_updates(None, None)?
                        .into_iter()
                        .filter_map(parse_update)
                        .filter_map(map_command_to_contract)
                        .collect::<Vec<_>>();
                    for command in commands {
                        println!("Command: {:?}", command);
                    }
                }
                "read-purge" => {
                    let mut offset = None;
                    let mut count = 0;
                    loop {
                        let updates = get_updates(offset, None)?;
                        if updates.is_empty() {
                            break;
                        }

                        count += updates.len();

                        let mut highest_update_id = 0;
                        for update in &updates {
                            if update.update_id > highest_update_id {
                                highest_update_id = update.update_id;
                            }

                            println!("purging #{}", update.update_id);
                        }

                        offset = Some(highest_update_id + 1);
                    }
                    println!("purged {} updates", count);
                }
                _ => {
                    return Err(anyhow::anyhow!("Unknown command: {data}"));
                }
            }
            Ok(None)
        }
        TriggerData::Cron(_) => {
            // Try to acquire the lock
            match acquire_lock() {
                Ok(lock_cas) => {
                    let command = get_next_command();
                    // Release the lock when done even if we got an error
                    release_lock(lock_cas)?;

                    match command? {
                        None => {
                            host::log(LogLevel::Info, "No new commands to process");
                            return Ok(None);
                        }
                        Some(command) => {
                            host::log(LogLevel::Warn, "GOT COMMAND!!!");
                            println!("{:#?}", command);

                            Ok(Some(WasmResponse {
                                payload: serde_json::to_vec(&command)?,
                                ordering: None,
                            }))
                        }
                    }
                }
                Err(_) => {
                    host::log(LogLevel::Warn, "Lock is already held by another component!");
                    // Lock is already held by another component
                    Ok(None)
                }
            }
        }
        _ => Ok(None),
    }
}

fn get_next_command() -> Result<Option<WavsPayload>> {
    let latest_offset: Option<i64> = get_offset()?;

    println!("LATEST OFFSET: {:?}", latest_offset);

    let update = match get_updates(latest_offset, Some(1))?.into_iter().next() {
        Some(update) => update,
        None => {
            return Ok(None);
        }
    };

    println!("UPDATE: {:?}", update);

    if let Err(e) = set_offset(update.update_id + 1) {
        host::log(
            LogLevel::Error,
            &format!("failed to set latest offset after getting update: {e:?}"),
        );
    }

    let message = match parse_update(update) {
        Some(msg) => msg,
        None => {
            host::log(LogLevel::Warn, "No valid message found in the update");
            return Ok(None);
        }
    };

    println!("MESSAGE: {:?}", message);

    Ok(map_command_to_contract(message))
}
