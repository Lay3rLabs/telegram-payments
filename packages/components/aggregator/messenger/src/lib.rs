use anyhow::Result;
use tg_components_shared::{ReportEvent, ReportEventRequest};
use wavs_wasi_utils::http::http_request_post_json;
use wstd::http::Client;

wit_bindgen::generate!({
    world: "aggregator-world",
    generate_all,
});

struct Component;

impl Guest for Component {
    fn process_packet(packet: Packet) -> Result<Vec<AggregatorAction>, String> {
        let secret = std::env::var("WAVS_ENV_SERVER_SECRET").unwrap_or_default();

        if secret.is_empty() {
            return Err(format!(
                "secret is not set in WAVS_ENV_SERVER_SECRET environment variable"
            ));
        }

        let endpoint = host::config_var("SERVER_ENDPOINT").unwrap_or_default();

        if endpoint.is_empty() {
            return Err(format!(
                "SERVER_ENDPOINT is not set in the component config"
            ));
        }

        let event: ReportEvent = serde_json::from_slice(&packet.envelope.payload)
            .map_err(|e| format!("Failed to deserialize packet payload: {}", e))?;

        let req = http_request_post_json(
            &endpoint,
            ReportEventRequest {
                secret,
                event_id: host::get_event_id(),
                event: event.clone(),
            },
        )
        .map_err(|s| s.to_string())?;

        wstd::runtime::block_on(async move {
            let client = Client::new();
            client.send(req).await
        })
        .map_err(|s| s.to_string())?;

        Ok(Vec::new())
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
