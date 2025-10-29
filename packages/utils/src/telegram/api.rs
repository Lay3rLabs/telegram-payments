use layer_climb::prelude::CosmosAddr;
use serde::{Deserialize, Serialize};

use crate::telegram::error::TelegramBotError;

#[derive(Clone, Debug)]
pub enum TelegramBotCommand {
    Start,
    Wavs(TelegramWavsCommand),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TelegramWavsCommand {
    Help,
    Receive {
        address: CosmosAddr,
        user_id: i64,
        user_name: Option<String>,
    },
    Send {
        handle: String,
        amount: u64,
        user_id: i64,
        user_name: Option<String>,
    },
    Status,
}

impl TryFrom<&TelegramMessage> for TelegramBotCommand {
    type Error = TelegramBotError;

    fn try_from(message: &TelegramMessage) -> Result<Self, Self::Error> {
        match message.text.clone() {
            Some(text) => {
                let parts: Vec<&str> = text.split_whitespace().collect();

                if parts.len() > 0 {
                    tracing::info!("PARTS: {:?}", parts);
                    match message.chat.chat_type {
                        TelegramChatType::Private | TelegramChatType::Group => match parts[..] {
                            ["/start"] => Ok(TelegramBotCommand::Start),
                            ["/help"] => Ok(TelegramBotCommand::Wavs(TelegramWavsCommand::Help)),
                            ["/status"] => {
                                Ok(TelegramBotCommand::Wavs(TelegramWavsCommand::Status))
                            }
                            ["/send", handle, amount] => {
                                Ok(TelegramBotCommand::Wavs(TelegramWavsCommand::Send {
                                    user_id: message.from.id,
                                    user_name: message.from.username.clone(),
                                    handle: handle.to_string(),
                                    amount: amount.parse().map_err(|e| {
                                        TelegramBotError::Parse(format!(
                                            "could not parse {text}: {e:?}"
                                        ))
                                    })?,
                                }))
                            }
                            ["/receive", address] => {
                                Ok(TelegramBotCommand::Wavs(TelegramWavsCommand::Receive {
                                    user_id: message.from.id,
                                    user_name: message.from.username.clone(),
                                    address: address.parse().map_err(|e| {
                                        TelegramBotError::Parse(format!(
                                            "could not parse {text}: {e:?}"
                                        ))
                                    })?,
                                }))
                            }
                            _ => Err(TelegramBotError::BadCommand(text)),
                        },
                        TelegramChatType::SuperGroup | TelegramChatType::Channel => {
                            Err(TelegramBotError::BadCommand(text))
                        }
                    }
                } else {
                    Err(TelegramBotError::UnknownCommand(text))
                }
            }
            None => Err(TelegramBotError::EmptyMessage),
        }
    }
}

pub struct TelegramWebHook {}

// https://core.telegram.org/bots/api#update
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramWebHookRequest {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
    pub edited_message: Option<TelegramMessage>,
    pub channel_post: Option<TelegramMessage>,
    pub edited_channel_post: Option<TelegramMessage>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramWebHookResponse {
    pub chat_id: i64,
    pub method: TelegramResponseMethod,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<String>,
}
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum TelegramResponseMethod {
    #[serde(rename = "sendMessage")]
    SendMessge,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub message_thread_id: Option<i64>,
    pub from: TelegramUser,
    pub chat: TelegramChat,
    pub date: u64,
    pub text: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramUser {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub username: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: TelegramChatType,
    pub title: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum TelegramChatType {
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "group")]
    Group,
    #[serde(rename = "supergroup")]
    SuperGroup,
    #[serde(rename = "channel")]
    Channel,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramWebHookInfo {
    pub url: String,
    pub has_custom_certificate: bool,
    pub pending_update_count: u64,
    pub ip_address: Option<String>,
    pub last_error_date: Option<u64>,
    pub last_error_message: Option<String>,
    pub last_synchronization_error_date: Option<u64>,
    pub max_connections: Option<u64>,
    pub allowed_updates: Option<Vec<String>>,
}

// https://core.telegram.org/bots/api#update
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramUpdate {
    pub update_id: i64,
    // Message updates
    pub message: Option<TelegramMessage>,
    pub edited_message: Option<TelegramMessage>,
    pub channel_post: Option<TelegramMessage>,
    pub edited_channel_post: Option<TelegramMessage>,
    // Business updates
    pub business_connection: Option<TelegramBusinessConnection>,
    pub business_message: Option<TelegramMessage>,
    pub edited_business_message: Option<TelegramMessage>,
    pub deleted_business_messages: Option<TelegramBusinessMessagesDeleted>,
    // Inline updates
    pub inline_query: Option<TelegramInlineQuery>,
    pub chosen_inline_result: Option<TelegramChosenInlineResult>,
    pub callback_query: Option<TelegramCallbackQuery>,
    // Payment updates
    pub shipping_query: Option<TelegramShippingQuery>,
    pub pre_checkout_query: Option<TelegramPreCheckoutQuery>,
    pub purchased_paid_media: Option<TelegramPaidMediaPurchased>,
    // Poll updates
    pub poll: Option<TelegramPoll>,
    pub poll_answer: Option<TelegramPollAnswer>,
    // Chat member updates
    pub my_chat_member: Option<TelegramChatMemberUpdated>,
    pub chat_member: Option<TelegramChatMemberUpdated>,
    pub chat_join_request: Option<TelegramChatJoinRequest>,
    // Boost updates
    pub chat_boost: Option<TelegramChatBoostUpdated>,
    pub removed_chat_boost: Option<TelegramChatBoostRemoved>,
    // Reaction updates
    pub message_reaction: Option<TelegramMessageReactionUpdated>,
    pub message_reaction_count: Option<TelegramMessageReactionCountUpdated>,
}

// https://core.telegram.org/bots/api#callbackquery
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramCallbackQuery {
    pub id: String,
    pub from: TelegramUser,
    pub message: Option<TelegramMessage>,
    pub inline_message_id: Option<String>,
    pub chat_instance: String,
    pub data: Option<String>,
    pub game_short_name: Option<String>,
}

// https://core.telegram.org/bots/api#inlinequery
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramInlineQuery {
    pub id: String,
    pub from: TelegramUser,
    pub query: String,
    pub offset: String,
    pub chat_type: Option<String>,
    pub location: Option<TelegramLocation>,
}

// https://core.telegram.org/bots/api#choseninlineresult
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramChosenInlineResult {
    pub result_id: String,
    pub from: TelegramUser,
    pub location: Option<TelegramLocation>,
    pub inline_message_id: Option<String>,
    pub query: String,
}

// https://core.telegram.org/bots/api#location
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub horizontal_accuracy: Option<f64>,
    pub live_period: Option<u64>,
    pub heading: Option<u64>,
    pub proximity_alert_radius: Option<u64>,
}

// https://core.telegram.org/bots/api#poll
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramPoll {
    pub id: String,
    pub question: String,
    pub question_entities: Option<Vec<TelegramMessageEntity>>,
    pub options: Vec<TelegramPollOption>,
    pub total_voter_count: u64,
    pub is_closed: bool,
    pub is_anonymous: bool,
    #[serde(rename = "type")]
    pub poll_type: String,
    pub allows_multiple_answers: bool,
    pub correct_option_id: Option<u64>,
    pub explanation: Option<String>,
    pub explanation_entities: Option<Vec<TelegramMessageEntity>>,
    pub open_period: Option<u64>,
    pub close_date: Option<u64>,
}

// https://core.telegram.org/bots/api#polloption
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramPollOption {
    pub text: String,
    pub text_entities: Option<Vec<TelegramMessageEntity>>,
    pub voter_count: u64,
}

// https://core.telegram.org/bots/api#pollanswer
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramPollAnswer {
    pub poll_id: String,
    pub voter_chat: Option<TelegramChat>,
    pub user: Option<TelegramUser>,
    pub option_ids: Vec<u64>,
}

// https://core.telegram.org/bots/api#messageentity
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramMessageEntity {
    #[serde(rename = "type")]
    pub entity_type: String,
    pub offset: u64,
    pub length: u64,
    pub url: Option<String>,
    pub user: Option<TelegramUser>,
    pub language: Option<String>,
    pub custom_emoji_id: Option<String>,
}

// Stub types for business features
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramBusinessConnection {
    pub id: String,
    pub user: TelegramUser,
    pub user_chat_id: i64,
    pub date: u64,
    pub can_reply: bool,
    pub is_enabled: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramBusinessMessagesDeleted {
    pub business_connection_id: String,
    pub chat: TelegramChat,
    pub message_ids: Vec<i64>,
}

// Stub types for payment features
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramShippingQuery {
    pub id: String,
    pub from: TelegramUser,
    pub invoice_payload: String,
    pub shipping_address: TelegramShippingAddress,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramShippingAddress {
    pub country_code: String,
    pub state: String,
    pub city: String,
    pub street_line1: String,
    pub street_line2: String,
    pub post_code: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramPreCheckoutQuery {
    pub id: String,
    pub from: TelegramUser,
    pub currency: String,
    pub total_amount: i64,
    pub invoice_payload: String,
    pub shipping_option_id: Option<String>,
    pub order_info: Option<TelegramOrderInfo>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramOrderInfo {
    pub name: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub shipping_address: Option<TelegramShippingAddress>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramPaidMediaPurchased {
    pub from: TelegramUser,
    pub paid_media_payload: String,
}

// Stub types for chat member features
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramChatMemberUpdated {
    pub chat: TelegramChat,
    pub from: TelegramUser,
    pub date: u64,
    pub old_chat_member: TelegramChatMember,
    pub new_chat_member: TelegramChatMember,
    pub invite_link: Option<TelegramChatInviteLink>,
    pub via_join_request: Option<bool>,
    pub via_chat_folder_invite_link: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramChatMember {
    pub status: String,
    pub user: TelegramUser,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramChatInviteLink {
    pub invite_link: String,
    pub creator: TelegramUser,
    pub creates_join_request: bool,
    pub is_primary: bool,
    pub is_revoked: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramChatJoinRequest {
    pub chat: TelegramChat,
    pub from: TelegramUser,
    pub user_chat_id: i64,
    pub date: u64,
    pub bio: Option<String>,
    pub invite_link: Option<TelegramChatInviteLink>,
}

// Stub types for boost features
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramChatBoostUpdated {
    pub chat: TelegramChat,
    pub boost: TelegramChatBoost,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramChatBoost {
    pub boost_id: String,
    pub add_date: u64,
    pub expiration_date: u64,
    pub source: TelegramChatBoostSource,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramChatBoostSource {
    pub source: String,
    pub user: Option<TelegramUser>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramChatBoostRemoved {
    pub chat: TelegramChat,
    pub boost_id: String,
    pub remove_date: u64,
    pub source: TelegramChatBoostSource,
}

// Stub types for reaction features
#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramMessageReactionUpdated {
    pub chat: TelegramChat,
    pub message_id: i64,
    pub user: Option<TelegramUser>,
    pub actor_chat: Option<TelegramChat>,
    pub date: u64,
    pub old_reaction: Vec<TelegramReactionType>,
    pub new_reaction: Vec<TelegramReactionType>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramMessageReactionCountUpdated {
    pub chat: TelegramChat,
    pub message_id: i64,
    pub date: u64,
    pub reactions: Vec<TelegramReactionCount>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramReactionType {
    #[serde(rename = "type")]
    pub reaction_type: String,
    pub emoji: Option<String>,
    pub custom_emoji_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramReactionCount {
    #[serde(rename = "type")]
    pub reaction_type: TelegramReactionType,
    pub total_count: u64,
}
