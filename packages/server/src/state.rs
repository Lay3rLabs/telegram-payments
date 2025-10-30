use anyhow::anyhow;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tg_utils::{
    client::payments::PaymentsQuerier,
    config::load_chain_configs_from_wavs,
    path::repo_root,
    telegram::{
        api::native::TelegramMessage,
        error::TgResult,
        messenger::{any_client::TelegramMessengerExt, reqwest_client::TelegramMessenger},
    },
};

use layer_climb::prelude::*;
use wavs_types::{ChainConfigs, ChainKey};

#[derive(Clone)]
pub struct HttpState {
    chain_configs: ChainConfigs,
    service: Arc<std::sync::Mutex<Option<wavs_types::Service>>>,
    query_clients: Arc<std::sync::Mutex<HashMap<ChainKey, QueryClient>>>,
}

impl HttpState {
    pub async fn new() -> Self {
        let chain_configs = load_chain_configs_from_wavs(None as Option<PathBuf>)
            .await
            .unwrap();

        Self {
            chain_configs,
            service: Arc::new(std::sync::Mutex::new(None)),
            query_clients: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
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

    pub async fn set_service(&self, url: &String) -> anyhow::Result<wavs_types::Service> {
        let service: wavs_types::Service = reqwest::get(url).await?.json().await?;

        *self.service.lock().unwrap() = Some(service.clone());

        Ok(service)
    }

    pub fn get_service(&self) -> Option<wavs_types::Service> {
        self.service.lock().unwrap().clone()
    }

    pub fn service_manager_chain(&self) -> anyhow::Result<Option<ChainKey>> {
        let service = match self.get_service() {
            Some(s) => s,
            None => {
                return Ok(None);
            }
        };

        let chain = match service.manager {
            wavs_types::ServiceManager::Cosmos { chain, .. } => chain,
            _ => {
                return Err(anyhow!("Service is not cosmos..."));
            }
        };

        Ok(Some(chain))
    }

    pub fn service_manager_address(&self) -> anyhow::Result<Option<CosmosAddr>> {
        let service = match self.get_service() {
            Some(s) => s,
            None => {
                return Ok(None);
            }
        };

        let address = match service.manager {
            wavs_types::ServiceManager::Cosmos { address, .. } => address,
            _ => {
                return Err(anyhow!("Service is not cosmos..."));
            }
        };

        Ok(Some(address))
    }

    pub fn payments_contract_address(&self) -> anyhow::Result<Option<CosmosAddr>> {
        let service = match self.get_service() {
            Some(s) => s,
            None => {
                return Ok(None);
            }
        };

        let address =
            service
                .workflows
                .values()
                .next()
                .and_then(|workflow| match &workflow.submit {
                    wavs_types::Submit::None => None,
                    wavs_types::Submit::Aggregator { component, .. } => {
                        component.config.get("PAYMENTS_CONTRACT_ADDRESS").cloned()
                    }
                });

        match address {
            Some(address) => Ok(Some(CosmosAddr::new_str(&address, None)?)),
            None => Ok(None),
        }
    }

    pub async fn get_service_uri(&self) -> anyhow::Result<Option<String>> {
        let address = match self.service_manager_address()? {
            Some(a) => a,
            None => {
                return Ok(None);
            }
        };
        let query_client = self.get_query_client().await?;

        let service_uri:String = query_client.contract_smart(&address.into(), &wavs_types::contracts::cosmwasm::service_manager::ServiceManagerQueryMessages::WavsServiceUri {  }).await?;

        Ok(Some(service_uri))
    }

    pub async fn get_query_client(&self) -> anyhow::Result<QueryClient> {
        let chain = self
            .service_manager_chain()?
            .ok_or_else(|| anyhow!("Service manager chain not set"))?;

        let client = { self.query_clients.lock().unwrap().get(&chain).cloned() };

        match client {
            Some(q) => Ok(q),
            None => {
                let config = self
                    .chain_configs
                    .get_chain(&chain)
                    .ok_or_else(|| anyhow!("Chain config not found for {}", chain))?;
                let config = match config {
                    wavs_types::AnyChainConfig::Cosmos(c) => c,
                    _ => {
                        return Err(anyhow!("Chain config is not cosmos..."));
                    }
                };
                let client = QueryClient::new(config.into(), None).await?;
                self.query_clients
                    .lock()
                    .unwrap()
                    .insert(chain.clone(), client.clone());
                Ok(client)
            }
        }
    }
}

pub struct TelegramBot {
    messenger: TelegramMessenger,
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

    pub async fn generate_group_invite_link(&self) -> TgResult<String> {
        self.messenger
            .generate_group_invite_link(self.group_id)
            .await
    }
}
