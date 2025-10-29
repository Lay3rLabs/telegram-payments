use async_trait::async_trait;
use std::collections::HashMap;

use crate::telegram::{
    error::{TelegramBotError, TgResult},
    messenger::any_client::TelegramMessengerExt,
};

pub struct TelegramMessenger {
    pub token: String,
    pub client: reqwest::Client,
}

#[async_trait]
impl TelegramMessengerExt for TelegramMessenger {
    fn token(&self) -> &str {
        &self.token
    }

    async fn fetch_string(&self, url: &str) -> TgResult<String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| TelegramBotError::Internal(format!("HTTP GET error: {}", e)))?;

        response
            .text()
            .await
            .map_err(|e| TelegramBotError::Internal(format!("Failed to read response: {}", e)))
    }

    async fn fetch_params(&self, url: &str, params: &HashMap<String, String>) -> TgResult<String> {
        let response = self
            .client
            .post(url)
            .form(params)
            .send()
            .await
            .map_err(|e| TelegramBotError::Internal(format!("HTTP POST error: {}", e)))?;

        response
            .text()
            .await
            .map_err(|e| TelegramBotError::Internal(format!("Failed to read response: {}", e)))
    }
}

impl TelegramMessenger {
    pub fn new(token: String, client: reqwest::Client) -> Self {
        Self { token, client }
    }
}
