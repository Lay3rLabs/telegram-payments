// #[allow(clippy::all)]
// mod bindings;

// use std::str::FromStr;

// use wavs_types::ChainKey;
// use wavs_wasi_utils::evm::alloy_primitives;

// use crate::bindings::{
//     host,
//     wavs::{
//         aggregator::aggregator::{CosmosSubmitAction, EvmAddress, EvmSubmitAction, SubmitAction},
//         types::chain::CosmosAddress,
//     },
//     AggregatorAction, AnyTxHash, Guest, Packet,
// };

// struct Component;
// impl Guest for Component {
//     fn process_packet(_pkt: Packet) -> Result<Vec<AggregatorAction>, String> {
//         let chain = host::config_var("chain").ok_or("chain config variable is required")?;
//         let service_handler = host::config_var("service_handler")
//             .ok_or("service_handler config variable is required")?;
//         let service_handler = layer_climb::prelude::CosmosAddr::new_str(&service_handler_str, None)
//             .map_err(|e| e.to_string())?;

//         let submit_action = SubmitAction::Cosmos(CosmosSubmitAction {
//             chain: chain.to_string(),
//             address: CosmosAddress {
//                 bech32_addr: service_handler.to_string(),
//                 prefix_len: service_handler.prefix().len() as u32,
//             },
//             gas_price: None,
//         });

//         Ok(vec![AggregatorAction::Submit(submit_action)])
//     }

//     fn handle_timer_callback(_packet: Packet) -> Result<Vec<AggregatorAction>, String> {
//         Err("Not implemented yet".to_string())
//     }

//     fn handle_submit_callback(
//         _packet: Packet,
//         tx_result: Result<AnyTxHash, String>,
//     ) -> Result<(), String> {
//         match tx_result {
//             Ok(_) => Ok(()),
//             Err(_) => Ok(()),
//         }
//     }
// }

// crate::bindings::export!(Component with_types_in crate::bindings);
