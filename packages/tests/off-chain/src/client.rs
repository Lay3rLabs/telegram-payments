//! Abstraction specifically for the off-chain multi-test environment
pub mod payments;
use std::{cell::RefCell, rc::Rc};
use tg_utils::client::{AnyExecutor, AnyQuerier};

use cosmwasm_std::{Addr, Coin};
use cw_multi_test::App;

#[derive(Clone)]
pub struct AppClient {
    pub querier: AnyQuerier,
    pub executor: AnyExecutor,
}

impl AppClient {
    pub fn new(admin: &str) -> Self {
        let app = Rc::new(RefCell::new(App::new(|router, api, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &api.addr_make(admin),
                    vec![Coin {
                        denom: "utoken".to_string(),
                        amount: 1_000_000u128.into(),
                    }],
                )
                .unwrap();
        })));

        let admin = app.borrow().api().addr_make(admin);

        Self {
            querier: app.clone().into(),
            executor: (app.clone(), admin).into(),
        }
    }

    pub fn app(&self) -> Rc<RefCell<App>> {
        match &self.executor {
            AnyExecutor::MultiTest { app, .. } => app.clone(),
            _ => unreachable!(),
        }
    }

    pub fn admin(&self) -> Addr {
        match &self.executor {
            AnyExecutor::MultiTest { admin, .. } => admin.clone(),
            _ => unreachable!(),
        }
    }
}
