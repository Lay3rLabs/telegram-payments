/// These are types for the state machine of parsing commands.
/// Only contains intermediate state, we get:
/// (State, Text) -> (State, Option<Command>)
#[derive(Clone, Debug)]
pub enum TGChatState {
    Wait,
    WavsReceive,
    WavsSend,
    WavsSendHandle(String),
    WavsSendHandleAmount(String, u64),
}

impl TGChatState {
    // Prompt for the next step, or None
    pub fn prompt(&self) -> Option<String> {
        match self {
            TGChatState::Wait => None,
            TGChatState::WavsReceive => {
                Some("What blockchain address would you like to receive to?".to_string())
            }
            TGChatState::WavsSend => Some("Who would you like to send to?".to_string()),
            TGChatState::WavsSendHandle(handle) => {
                Some(format!("How much would you like to send to {}?", handle))
            }
            TGChatState::WavsSendHandleAmount(_, _) => Some("Which denom?".to_string()),
        }
    }

    pub fn next_state(self, text: &str) -> anyhow::Result<(Self, Option<TelegramWavsCommand>)> {
        match self {
            Self::Wait => match text {
                "/start" => Ok((TGChatState::Wait, Some(TelegramWavsCommand::Start))),
                "/help" => Ok((TGChatState::Wait, Some(TelegramWavsCommand::Help))),
                "/status" => Ok((TGChatState::Wait, Some(TelegramWavsCommand::Status))),
                "/send" => Ok((TGChatState::WavsSend, None)),
                "/receive" => Ok((TGChatState::WavsReceive, None)),
                x => bail!("unknown command: {x}"),
            },
            Self::WavsReceive => {
                if text.starts_with("/") {
                    return Self::Wait.next_state(text);
                }
                let address = text.parse::<CosmosAddr>()?;
                Ok((Self::Wait, Some(TelegramWavsCommand::Receive { address })))
            }
            TGChatState::WavsSend => {
                if text.starts_with("/") {
                    return Self::Wait.next_state(text);
                }
                // get handle
                let handle = text.trim();
                if !handle.starts_with("@") {
                    bail!("Provide a telegram username, starting with @");
                }
                Ok((TGChatState::WavsSendHandle(handle.to_string()), None))
            }
            TGChatState::WavsSendHandle(handle) => {
                if text.starts_with("/") {
                    return Self::Wait.next_state(text);
                }
                // get amount
                let amount: u64 = text.trim().parse()?;
                Ok((Self::WavsSendHandleAmount(handle.to_string(), amount), None))
            }
            TGChatState::WavsSendHandleAmount(handle, amount) => {
                if text.starts_with("/") {
                    return Self::Wait.next_state(text);
                }
                // get denom
                let denom = text.trim();
                Ok((
                    Self::Wait,
                    Some(TelegramWavsCommand::Send {
                        handle: handle.to_string(),
                        amount,
                        denom: denom.to_string(),
                    }),
                ))
            }
        }
    }
}
