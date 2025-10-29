mod entry;
mod tg_helpers;

// this is needed just to make the ide/compiler happy... we're _always_ compiling to wasm32-wasi
wit_bindgen::generate!({
    world: "wavs-world",
    generate_all,
});

struct Component;

impl Guest for Component {
    fn run(trigger_action: TriggerAction) -> Result<Option<WasmResponse>, String> {
        entry::handle_action(trigger_action).map_err(|e| e.to_string())
    }
}

export!(Component);
