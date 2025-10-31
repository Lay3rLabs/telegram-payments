use crate::state::HttpState;
use axum::extract::{Json, State};
use axum::response::IntoResponse;
#[cfg(debug_assertions)]
use tg_components_shared::ReportEventRequest;

#[cfg(debug_assertions)]
#[axum::debug_handler]
pub async fn handle_tg_component(
    State(state): State<HttpState>,
    Json(req): Json<ReportEventRequest>,
) -> impl IntoResponse {
    use tg_components_shared::ReportEvent;
    use tg_contract_api::payments::event::{ConnectEvent, RegistrationEvent, SendPaymentEvent};

    use crate::error::AnyError;

    if req.secret != state.component_secret {
        tracing::warn!("Invalid secret in telegram component report");
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    }

    // hacky but fine for now :P
    if !state.should_send_event_id(req.event_id.clone()) {
        tracing::info!(
            "Event ID {:?} has already been processed, skipping",
            req.event_id
        );
        return axum::http::StatusCode::OK.into_response();
    }

    let text = match req.event {
        ReportEvent::Connect(ConnectEvent { tg_handle, address }) => {
            format!(
                "User connected!\nTelegram: @{}\nAddress: {}",
                tg_handle, address
            )
        }
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

    match state.tg_bot().send_message_to_group(&text).await {
        Ok(_) => axum::http::StatusCode::OK.into_response(),
        Err(e) => {
            tracing::error!("Failed to send telegram message: {:?}", e);
            AnyError::from(e).into_response()
        }
    }
}
