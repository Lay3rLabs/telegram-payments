use anyhow::Result;
use tg_components_shared::ReportEvent;
use tg_contract_api::payments::event::{RegistrationEvent, SendPaymentEvent};
use tg_utils::telegram::messenger::{
    any_client::TelegramMessengerExt, wasi_client::TelegramMessenger,
};

wit_bindgen::generate!({
    world: "aggregator-world",
    generate_all,
});

struct Component;

impl Guest for Component {
    fn process_packet(packet: Packet) -> Result<Vec<AggregatorAction>, String> {
        let bot_token = std::env::var("WAVS_ENV_AGGREGATOR_TELEGRAM_BOT_TOKEN").unwrap_or_default();

        if bot_token.is_empty() {
            return Err(format!(
                "BOT TOKEN is not set in WAVS_ENV_AGGREGATOR_TELEGRAM_BOT_TOKEN"
            ));
        }

        let group_id = host::config_var("TELEGRAM_GROUP_ID").unwrap_or_default();

        if group_id.is_empty() {
            return Err(format!(
                "TELEGRAM_GROUP_ID is not set in the component config"
            ));
        }

        let group_id = group_id
            .parse::<i64>()
            .map_err(|e| format!("Failed to parse TELEGRAM_GROUP_ID: {}", e))?;

        let event: ReportEvent = serde_json::from_slice(&packet.envelope.payload)
            .map_err(|e| format!("Failed to deserialize packet payload: {}", e))?;

        let text = match event {
            ReportEvent::Registration(RegistrationEvent { tg_handle, address }) => {
                format!(
                    "New user registered!\nTelegram: @{}\nAddress: {}",
                    tg_handle, address
                )
            }

            ReportEvent::SendPayment(SendPaymentEvent {
                from_tg_handle,
                to_tg_handle,
                from_address,
                to_address,
                amount,
                denom,
            }) => {
                format!("Payment sent!\nFrom: @{from_tg_handle} ({from_address})\nTo: @{to_tg_handle} ({to_address})\nAmount: {amount} {denom}")
            }
        };

        let tg_messenger = TelegramMessenger::new(bot_token);

        println!("Sending Telegram message to group {}: {}", group_id, text);
        wstd::runtime::block_on(async move {
            tg_messenger
                .send_message(group_id, &text)
                .await
                .map_err(|e| format!("Failed to send Telegram message: {}", e))
        })?;

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
