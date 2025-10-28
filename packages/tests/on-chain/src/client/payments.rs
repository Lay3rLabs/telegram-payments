use layer_climb::prelude::Address;
use tg_utils::client::payments::{PaymentsExecutor, PaymentsQuerier};

use crate::{client::AppClient, code_ids::CodeId};

#[derive(Clone)]
pub struct PaymentsClient {
    pub querier: PaymentsQuerier,
    pub executor: PaymentsExecutor,
}

impl PaymentsClient {
    pub async fn new(app_client: AppClient, admin: Option<Address>) -> Self {
        let pool = app_client.pool();
        let client = pool.get().await.unwrap();

        let admin = admin.unwrap_or_else(|| client.addr.clone());

        let msg = tg_contract_api::payments::msg::InstantiateMsg {
            allowed_denoms: vec!["untrn".to_string(), "uatom".to_string()],
            auth: tg_contract_api::payments::msg::Auth::Admin(admin.to_string()),
        };

        let (address, _) = client
            .contract_instantiate(
                None,
                CodeId::new_payments().await,
                "Telegram payments",
                &msg,
                vec![],
                None,
            )
            .await
            .unwrap();

        let querier = PaymentsQuerier::new(app_client.querier.clone(), address.clone().into());
        let executor = PaymentsExecutor::new(client.clone().into(), address.clone().into());

        Self { querier, executor }
    }
}
