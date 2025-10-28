mod entry;

wit_bindgen::generate!({
    world: "wavs-world",
    generate_all,
});

struct Component;

impl Guest for Component {
    fn run(_trigger_action: TriggerAction) -> Result<Option<WasmResponse>, _rt::String> {
        Ok(None)
    }
}

export!(Component);
