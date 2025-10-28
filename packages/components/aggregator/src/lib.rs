mod entry;

wit_bindgen::generate!({
    world: "aggregator-world",
    generate_all,
});

struct Component;

impl Guest for Component {
    fn process_packet(_packet: Packet) -> Result<Vec<AggregatorAction>, String> {
        Ok(vec![])
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
