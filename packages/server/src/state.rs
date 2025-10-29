use tg_utils::telegram::{api::TelegramMessage, error::TgResult, messenger::TelegramMessenger};

#[derive(Clone, Default)]
pub struct HttpState {}

impl HttpState {
    pub fn new() -> Self {
        Self {}
    }

    // lazy loaded in case we're bootstrapping
    pub fn tg_bot(&self) -> TelegramBot {
        let bot_token = std::env::var("SERVER_TELEGRAM_BOT_TOKEN").unwrap_or_default();
        if bot_token.is_empty() {
            panic!("SERVER_TELEGRAM_BOT_TOKEN is not set");
        }
        let group_id = std::env::var("SERVER_TELEGRAM_GROUP_ID").unwrap_or_default();
        if bot_token.is_empty() {
            panic!("SERVER_TELEGRAM_GROUP_ID is not set");
        }

        TelegramBot::new(bot_token, group_id.parse().expect("Invalid group id"))
    }
}

pub struct TelegramBot {
    messenger: TelegramMessenger<reqwest::Client>,
    group_id: i64,
}

impl TelegramBot {
    pub fn new(token: String, group_id: i64) -> Self {
        Self {
            messenger: TelegramMessenger::new(token, reqwest::Client::new()),
            group_id,
        }
    }

    pub async fn send_message_to_group(&self, text: &str) -> TgResult<TelegramMessage> {
        self.messenger.send_message(self.group_id, text).await
    }
}
