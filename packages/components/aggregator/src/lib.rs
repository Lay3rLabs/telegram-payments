mod entry;

wit_bindgen::generate!({
    world: "aggregator-world",
    generate_all,
});

struct Component;

impl Guest for Component {
    fn process_packet(_packet: Packet) -> Result<_rt::Vec<AggregatorAction>, _rt::String> {
        todo!()
    }

    fn handle_timer_callback(_packet: Packet) -> Result<_rt::Vec<AggregatorAction>, _rt::String> {
        todo!()
    }

    fn handle_submit_callback(
        _packet: Packet,
        _tx_result: Result<AnyTxHash, _rt::String>,
    ) -> Result<(), _rt::String> {
        todo!()
    }
}

export!(Component);
