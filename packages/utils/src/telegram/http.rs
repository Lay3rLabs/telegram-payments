use crate::telegram::error::*;
use std::collections::HashMap;

#[cfg(feature = "reqwest")]
async fn reqwest_get(client: reqwest::Client, url: &str) -> TgResult<String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| TelegramBotError::Internal(format!("HTTP GET error: {}", e)))?;

    response
        .text()
        .await
        .map_err(|e| TelegramBotError::Internal(format!("Failed to read response: {}", e)))
}

#[cfg(feature = "wasi")]
async fn wasi_get(url: &str) -> TgResult<String> {
    let req = wavs_wasi_utils::http::http_request_get(url)
        .map_err(|e| TelegramBotError::Internal(format!("HTTP GET error: {}", e)))?;
    let text = wavs_wasi_utils::http::fetch_string(req)
        .await
        .map_err(|e| TelegramBotError::Internal(format!("HTTP GET error: {}", e)))?;

    Ok(text)
}

#[cfg(feature = "reqwest")]
async fn reqwest_post_form(
    client: reqwest::Client,
    url: &str,
    params: HashMap<String, String>,
) -> TgResult<String> {
    let response = client
        .post(url)
        .form(&params)
        .send()
        .await
        .map_err(|e| TelegramBotError::Internal(format!("HTTP POST error: {}", e)))?;

    response
        .text()
        .await
        .map_err(|e| TelegramBotError::Internal(format!("Failed to read response: {}", e)))
}

#[cfg(feature = "wasi")]
async fn wasi_post_form(url: &str, params: HashMap<String, String>) -> TgResult<String> {
    let req = wavs_wasi_utils::http::http_request_post_form(url, params)
        .map_err(|e| TelegramBotError::Internal(format!("HTTP POST error: {}", e)))?;

    let text = wavs_wasi_utils::http::fetch_string(req)
        .await
        .map_err(|e| TelegramBotError::Internal(format!("HTTP POST error: {}", e)))?;

    Ok(text)
}
