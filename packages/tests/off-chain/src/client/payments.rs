use cw_multi_test::{ContractWrapper, Executor};
use tg_utils::client::payments::{PaymentsExecutor, PaymentsQuerier};

use crate::client::AppClient;

#[derive(Clone)]
pub struct PaymentsClient {
    pub querier: PaymentsQuerier,
    pub executor: PaymentsExecutor,
}

impl PaymentsClient {
    pub fn new(app_client: AppClient) -> Self {
        let app = app_client.app();
        let admin = app_client.admin();

        let contract = ContractWrapper::new(
            tg_contract_payments::execute,
            tg_contract_payments::instantiate,
            tg_contract_payments::query,
        );
        let code_id = app.borrow_mut().store_code(Box::new(contract));

        let address = app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                admin.clone(),
                &tg_contract_api::payments::msg::InstantiateMsg {
                    allowed_denoms: vec![],
                    service_manager: "TODO".to_string(),
                },
                &[],
                "Payments",
                None,
            )
            .unwrap();

        let querier = PaymentsQuerier::new(app_client.querier.clone(), address.clone());
        let executor = PaymentsExecutor::new(app_client.executor.clone(), address);

        Self { querier, executor }
    }
}
