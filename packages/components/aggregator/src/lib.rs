use layer_climb::prelude::CosmosAddr;
use wavs_types::contracts::cosmwasm::service_handler::WavsEnvelope;

use crate::wavs::aggregator::aggregator::{CosmosAddress, CosmosSubmitAction, SubmitAction};

mod entry;

wit_bindgen::generate!({
    world: "aggregator-world",
    generate_all,
});

struct Component;

impl Guest for Component {
    fn process_packet(packet: Packet) -> Result<Vec<AggregatorAction>, String> {
        let chain = host::config_var("CHAIN").ok_or("CHAIN config var is required")?;
        let payments_addr = host::config_var("PAYMENTS_CONTRACT_ADDRESS")
            .ok_or("PAYMENTS_CONTRACT_ADDRESS config var is required")?;

        host::get_cosmos_chain_config(&chain)
            .ok_or(format!("failed to get chain config for {}", chain))?;

        let payments_addr = CosmosAddr::new_str(&payments_addr, None).map_err(|e| e.to_string())?;

        let envelope = WavsEnvelope {
            data: packet.envelope.clone(),
            sender: None,
        };
        let envelope_temp = packet
            .envelope
            .decode()
            .map_err(|e| ContractError::AbiDecode(e.to_string()))?;

        Ok(vec![AggregatorAction::Submit(SubmitAction::Cosmos(
            CosmosSubmitAction {
                chain: chain.to_string(),
                address: CosmosAddress {
                    bech32_addr: payments_addr.to_string(),
                    prefix_len: payments_addr.prefix().len() as u32,
                },
                gas_price: None,
            },
        ))])
    }

    fn handle_timer_callback(_packet: Packet) -> Result<Vec<AggregatorAction>, String> {
        Ok(vec![])
    }

    fn handle_submit_callback(
        _packet: Packet,
        _tx_result: Result<AnyTxHash, String>,
    ) -> Result<(), String> {
        Ok(())
    }
}

export!(Component);
