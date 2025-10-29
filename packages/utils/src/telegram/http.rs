use std::collections::HashMap;
use async_trait::async_trait;
use crate::telegram::error::TgResult;

/// Trait for making HTTP requests to the Telegram Bot API
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Perform a GET request and return the response body as text
    async fn get(&self, url: &str) -> TgResult<String>;

    /// Perform a POST request with form data and return the response body as text
    async fn post_form(&self, url: &str, params: HashMap<String, String>) -> TgResult<String>;
}

#[cfg(feature = "reqwest")]
mod reqwest_impl {
    use super::*;
    use crate::telegram::error::TelegramBotError;

    #[async_trait]
    impl HttpClient for reqwest::Client {
        async fn get(&self, url: &str) -> TgResult<String> {
            let response = self
                .get(url)
                .send()
                .await
                .map_err(|e| TelegramBotError::Internal(format!("HTTP GET error: {}", e)))?;

            response
                .text()
                .await
                .map_err(|e| TelegramBotError::Internal(format!("Failed to read response: {}", e)))
        }

        async fn post_form(&self, url: &str, params: HashMap<String, String>) -> TgResult<String> {
            let response = self
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
    }
}
