use async_trait::async_trait;

use std::collections::HashMap;

use crate::telegram::{error::*, messenger::any_client::TelegramMessengerExt};

pub struct TelegramMessenger {
    pub token: String,
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        #[async_trait(?Send)]
        impl TelegramMessengerExt for TelegramMessenger {
            fn token(&self) -> &str {
                &self.token
            }

            async fn fetch_string(&self, url: &str) -> TgResult<String> {
                let req = wavs_wasi_utils::http::http_request_get(url)
                    .map_err(|e| TelegramBotError::Internal(format!("HTTP GET error: {}", e)))?;

                let text = wavs_wasi_utils::http::fetch_string(req)
                    .await
                    .map_err(|e| TelegramBotError::Internal(format!("HTTP GET error: {}", e)))?;

                Ok(text)
            }

            async fn fetch_params(&self, url: &str, params: &HashMap<String, String>) -> TgResult<String> {
                let req = wavs_wasi_utils::http::http_request_post_form(url, params.clone())
                    .map_err(|e| TelegramBotError::Internal(format!("HTTP POST error: {}", e)))?;

                let text = wavs_wasi_utils::http::fetch_string(req)
                    .await
                    .map_err(|e| TelegramBotError::Internal(format!("HTTP POST error: {}", e)))?;

                Ok(text)
            }
        }
    } else {
        #[async_trait]
        impl TelegramMessengerExt for TelegramMessenger {
            fn token(&self) -> &str {
                &self.token
            }

            async fn fetch_string(&self, _url: &str) -> TgResult<String> {
                unreachable!("WASI client should not be used outside of wasm32 target")
            }

            async fn fetch_params(&self, _url: &str, _params: &HashMap<String, String>) -> TgResult<String> {
                unreachable!("WASI client should not be used outside of wasm32 target")
            }
        }

    }
}

impl TelegramMessenger {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}
