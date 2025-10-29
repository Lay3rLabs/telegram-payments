use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

use crate::telegram::{
    api::{TelegramMessage, TelegramUser, TelegramWebHookInfo},
    error::{TelegramBotError, TgResult},
};

pub struct TelegramMessenger {
    pub token: String,
}

impl TelegramMessenger {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    pub async fn get_me(&self) -> TgResult<TelegramUser> {
        self.make_request_empty("getMe").await
    }

    pub async fn set_webhook(&self, url: &str, secret: &str) -> TgResult<()> {
        let mut params = HashMap::new();
        params.insert("url".to_string(), url.to_string());
        params.insert("secret_token".to_string(), secret.to_string());
        params.insert("drop_pending_updates".to_string(), "True".to_string());

        let success = self
            .make_request_params::<bool>("setWebhook", params)
            .await?;

        if success {
            Ok(())
        } else {
            Err(TelegramBotError::Internal(
                "Failed to set webhook".to_string(),
            ))
        }
    }

    pub async fn get_webhook(&self) -> TgResult<TelegramWebHookInfo> {
        self.make_request_empty("getWebhookInfo").await
    }

    pub async fn send_message(&self, chat_id: i64, text: &str) -> TgResult<TelegramMessage> {
        let mut params = HashMap::new();
        params.insert("chat_id".to_string(), chat_id.to_string());
        params.insert("text".to_string(), text.to_string());

        self.make_request_params("sendMessage", params).await
    }

    pub async fn make_request_params<T: DeserializeOwned>(
        &self,
        method: &str,
        params: HashMap<String, String>,
    ) -> TgResult<T> {
        let url = format!("https://api.telegram.org/bot{}/{}", self.token, method);
        let req = wavs_wasi_utils::http::http_request_post_form(&url, params)
            .map_err(|e| TelegramBotError::Internal(format!("HTTP POST error: {}", e)))?;

        let text = wavs_wasi_utils::http::fetch_string(req)
            .await
            .map_err(|e| TelegramBotError::Internal(format!("HTTP GET error: {}", e)))?;

        tracing::info!("Response: {}", text);

        let json: TelegramResult<T> =
            serde_json::from_str(&text).map_err(|e| TelegramBotError::Internal(e.to_string()))?;

        if json.ok {
            Ok(json.result)
        } else {
            Err(TelegramBotError::Internal("Telegram API error".to_string()))
        }
    }

    pub async fn make_request_empty<T: DeserializeOwned>(&self, method: &str) -> TgResult<T> {
        let url = format!("https://api.telegram.org/bot{}/{}", self.token, method);
        let req = wavs_wasi_utils::http::http_request_get(&url)
            .map_err(|e| TelegramBotError::Internal(format!("HTTP GET error: {}", e)))?;

        let text = wavs_wasi_utils::http::fetch_string(req)
            .await
            .map_err(|e| TelegramBotError::Internal(format!("HTTP POST error: {}", e)))?;

        let json: TelegramResult<T> =
            serde_json::from_str(&text).map_err(|e| TelegramBotError::Internal(e.to_string()))?;

        if json.ok {
            Ok(json.result)
        } else {
            Err(TelegramBotError::Internal("Telegram API error".to_string()))
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct TelegramResult<T> {
    ok: bool,
    result: T,
}
