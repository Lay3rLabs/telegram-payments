use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

use crate::telegram::{
    api::native::{TelegramMessage, TelegramUpdate, TelegramUser, TelegramWebHookInfo},
    error::{TelegramBotError, TgResult},
};

// Works with WASI and Reqwest clients
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait TelegramMessengerExt {
    // impl these
    fn token(&self) -> &str;
    async fn fetch_string(&self, url: &str) -> TgResult<String>;
    async fn fetch_params(&self, url: &str, params: &HashMap<String, String>) -> TgResult<String>;

    // get the rest for free
    async fn get_me(&self) -> TgResult<TelegramUser> {
        self._make_request_empty("getMe").await
    }

    async fn generate_group_invite_link(&self, chat_id: i64) -> TgResult<String> {
        let mut params = HashMap::new();
        params.insert("chat_id".to_string(), chat_id.to_string());

        let invite_link = self
            ._make_request_params::<String>("exportChatInviteLink", params)
            .await?;

        Ok(invite_link)
    }

    async fn set_webhook(&self, url: &str, secret: &str) -> TgResult<()> {
        let mut params = HashMap::new();
        params.insert("url".to_string(), url.to_string());
        params.insert("secret_token".to_string(), secret.to_string());
        params.insert("drop_pending_updates".to_string(), "True".to_string());

        let success = self
            ._make_request_params::<bool>("setWebhook", params)
            .await?;

        if success {
            Ok(())
        } else {
            Err(TelegramBotError::Internal(
                "Failed to set webhook".to_string(),
            ))
        }
    }

    async fn get_webhook(&self) -> TgResult<TelegramWebHookInfo> {
        self._make_request_empty("getWebhookInfo").await
    }

    async fn get_updates(
        &self,
        offset: Option<i64>,
        limit: Option<u32>,
        timeout: Option<u32>,
        allowed_updates: Option<Vec<String>>,
    ) -> TgResult<Vec<TelegramUpdate>> {
        let mut params = HashMap::new();

        if let Some(offset) = offset {
            params.insert("offset".to_string(), offset.to_string());
        }

        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        if let Some(timeout) = timeout {
            params.insert("timeout".to_string(), timeout.to_string());
        }

        if let Some(allowed_updates) = allowed_updates {
            let json = serde_json::to_string(&allowed_updates)
                .map_err(|e| TelegramBotError::Internal(e.to_string()))?;
            params.insert("allowed_updates".to_string(), json);
        }

        if params.is_empty() {
            self._make_request_empty("getUpdates").await
        } else {
            self._make_request_params("getUpdates", params).await
        }
    }

    async fn send_miniapp_button(
        &self,
        chat_id: i64,
        label: &str,
        url: &str,
    ) -> TgResult<TelegramMessage> {
        let mut params = HashMap::new();
        params.insert("chat_id".to_string(), chat_id.to_string());
        params.insert("text".to_string(), label.to_string());
        let keyboard = serde_json::json!({
            "inline_keyboard": [[
                {
                    "text": "Open Mini App",
                    "web_app": {
                        "url": url
                    }
                }
            ]]
        });
        params.insert("reply_markup".to_string(), keyboard.to_string());

        self._make_request_params("sendMessage", params).await
    }

    async fn send_message(&self, chat_id: i64, text: &str) -> TgResult<TelegramMessage> {
        let mut params = HashMap::new();
        params.insert("chat_id".to_string(), chat_id.to_string());
        params.insert("text".to_string(), text.to_string());
        params.insert("parse_mode".to_string(), "Markdown".to_string());

        self._make_request_params("sendMessage", params).await
    }

    async fn _make_request_params<T: DeserializeOwned>(
        &self,
        method: &str,
        params: HashMap<String, String>,
    ) -> TgResult<T> {
        let url = format!("https://api.telegram.org/bot{}/{}", self.token(), method);

        let res = self.fetch_params(&url, &params).await;

        match &res {
            Ok(text) => println!("Response: {:?}", text),
            Err(e) => println!("Error fetching params: {:?}", e),
        }

        let text = res?;

        let json: TelegramResult<T> =
            serde_json::from_str(&text).map_err(|e| TelegramBotError::Internal(e.to_string()))?;

        if json.ok {
            Ok(json.result)
        } else {
            Err(TelegramBotError::Internal("Telegram API error".to_string()))
        }
    }

    async fn _make_request_empty<T: DeserializeOwned>(&self, method: &str) -> TgResult<T> {
        let url = format!("https://api.telegram.org/bot{}/{}", self.token(), method);

        let text = self.fetch_string(&url).await?;

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
