//! Abstractions for different backends (Climb, Climb Pool, MultiTest)
//! Provides AnyQuerier and AnyExecutor to represent _any_ contract querier/executor
//! The idea is that by moving the heavy-lifting here, we're free to write higher-level code
//! that provides an idiomatic and clean API
pub mod payments;

#[cfg(feature = "multitest")]
use cw_multi_test::{App, Executor};
#[cfg(feature = "multitest")]
use std::{cell::RefCell, rc::Rc};

use cosmwasm_std::Addr;
use layer_climb::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum AnyQuerier {
    Climb(QueryClient),
    #[cfg(feature = "climb_pool")]
    ClimbPool(layer_climb::pool::SigningClientPool),
    #[cfg(feature = "multitest")]
    MultiTest(Rc<RefCell<App>>),
}

impl From<QueryClient> for AnyQuerier {
    fn from(client: QueryClient) -> AnyQuerier {
        AnyQuerier::Climb(client)
    }
}

#[cfg(feature = "climb_pool")]
impl From<layer_climb::pool::SigningClientPool> for AnyQuerier {
    fn from(pool: layer_climb::pool::SigningClientPool) -> AnyQuerier {
        AnyQuerier::ClimbPool(pool)
    }
}

#[cfg(feature = "multitest")]
impl From<Rc<RefCell<App>>> for AnyQuerier {
    fn from(app: Rc<RefCell<App>>) -> AnyQuerier {
        AnyQuerier::MultiTest(app)
    }
}

impl AnyQuerier {
    pub async fn contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &Addr,
        msg: &MSG,
    ) -> Result<RESP, cosmwasm_std::StdError> {
        match self {
            Self::Climb(client) => {
                let addr = layer_climb::prelude::Address::try_from(address)
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
                client
                    .contract_smart(&addr, msg)
                    .await
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))
            }
            #[cfg(feature = "climb_pool")]
            Self::ClimbPool(pool) => {
                let addr = layer_climb::prelude::Address::try_from(address)
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
                let client = pool
                    .get()
                    .await
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
                client
                    .querier
                    .contract_smart(&addr, msg)
                    .await
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))
            }
            #[cfg(feature = "multitest")]
            Self::MultiTest(app) => app.borrow().wrap().query_wasm_smart(address, msg),
        }
    }
}

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum AnyExecutor {
    Climb(SigningClient),
    #[cfg(feature = "climb_pool")]
    ClimbPool(layer_climb::pool::SigningClientPool),
    #[cfg(feature = "multitest")]
    MultiTest {
        app: Rc<RefCell<App>>,
        admin: Addr,
    },
}

impl From<SigningClient> for AnyExecutor {
    fn from(client: SigningClient) -> AnyExecutor {
        AnyExecutor::Climb(client)
    }
}

#[cfg(feature = "climb_pool")]
impl From<layer_climb::pool::SigningClientPool> for AnyExecutor {
    fn from(pool: layer_climb::pool::SigningClientPool) -> AnyExecutor {
        AnyExecutor::ClimbPool(pool)
    }
}

#[cfg(feature = "multitest")]
impl From<(Rc<RefCell<App>>, Addr)> for AnyExecutor {
    fn from((app, admin): (Rc<RefCell<App>>, Addr)) -> AnyExecutor {
        AnyExecutor::MultiTest { app, admin }
    }
}

impl AnyExecutor {
    pub async fn contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &Addr,
        msg: &MSG,
        funds: &[cosmwasm_std::Coin],
    ) -> Result<AnyTxResponse, cosmwasm_std::StdError> {
        match self {
            Self::Climb(client) => {
                let addr = Address::try_from(address)
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
                let funds = funds
                    .iter()
                    .map(|c| layer_climb::prelude::Coin {
                        denom: c.denom.clone(),
                        amount: c.amount.to_string(),
                    })
                    .collect::<Vec<_>>();

                client
                    .contract_execute(&addr, msg, funds, None)
                    .await
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))
                    .map(AnyTxResponse::Climb)
            }
            #[cfg(feature = "climb_pool")]
            Self::ClimbPool(pool) => {
                let client = pool
                    .get()
                    .await
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
                let addr = Address::try_from(address)
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))?;
                let funds = funds
                    .iter()
                    .map(|c| layer_climb::prelude::Coin {
                        denom: c.denom.clone(),
                        amount: c.amount.to_string(),
                    })
                    .collect::<Vec<_>>();

                client
                    .contract_execute(&addr, msg, funds, None)
                    .await
                    .map_err(|e| cosmwasm_std::StdError::msg(e.to_string()))
                    .map(AnyTxResponse::Climb)
            }
            #[cfg(feature = "multitest")]
            Self::MultiTest { app, admin } => app
                .borrow_mut()
                .execute_contract(admin.clone(), address.clone(), msg, funds)
                .map(AnyTxResponse::MultiTest),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum AnyTxResponse {
    Climb(layer_climb::proto::abci::TxResponse),
    #[cfg(feature = "multitest")]
    MultiTest(cw_multi_test::AppResponse),
}

impl<'a> From<&'a AnyTxResponse> for CosmosTxEvents<'a> {
    fn from(value: &'a AnyTxResponse) -> Self {
        match value {
            AnyTxResponse::Climb(resp) => CosmosTxEvents::from(resp),
            #[cfg(feature = "multitest")]
            AnyTxResponse::MultiTest(resp) => CosmosTxEvents::from(resp.events.as_slice()),
        }
    }
}

impl AnyTxResponse {
    pub fn unchecked_into_tx_response(self) -> layer_climb::proto::abci::TxResponse {
        match self {
            Self::Climb(tx_resp) => tx_resp,
            #[allow(unreachable_patterns)]
            _ => panic!("unable to get unchecked tx response"),
        }
    }
}
