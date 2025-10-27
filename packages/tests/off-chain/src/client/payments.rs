use cosmwasm_std::Addr;
use cw_multi_test::{ContractWrapper, Executor};
use tg_utils::client::payments::{PaymentsExecutor, PaymentsQuerier};

use crate::client::AppClient;

#[derive(Clone)]
pub struct PaymentsClient {
    pub querier: PaymentsQuerier,
    pub executor: PaymentsExecutor,
}

impl PaymentsClient {
    pub fn new(app_client: AppClient, admin: Option<Addr>) -> Self {
        let admin = admin.unwrap_or(app_client.admin());

        let contract = ContractWrapper::new(
            tg_contract_payments::execute,
            tg_contract_payments::instantiate,
            tg_contract_payments::query,
        );
        let code_id = app_client.with_app_mut(|app| app.store_code(Box::new(contract)));

        let msg = tg_contract_api::payments::msg::InstantiateMsg {
            allowed_denoms: vec!["untrn".to_string(), "uatom".to_string()],
            auth: tg_contract_api::payments::msg::Auth::Admin(admin.to_string()),
        };

        let address = app_client.with_app_mut(|app| {
            app.instantiate_contract(code_id, admin.clone(), &msg, &[], "telegram-payments", None)
                .unwrap()
        });

        let querier = PaymentsQuerier::new(app_client.querier.clone(), address.clone());
        let executor = PaymentsExecutor::new(app_client.executor.clone(), address);

        Self { querier, executor }
    }
}
