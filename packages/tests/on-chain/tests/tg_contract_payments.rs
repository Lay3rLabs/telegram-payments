use on_chain_tests::client::{payments::PaymentsClient, AppClient};
use tg_test_common::shared_tests::{self, payments::RegisterReceivesOpenAccountProps};
use tg_utils::tracing::tracing_init;

#[tokio::test]
async fn get_admin() {
    tracing_init();

    let app_client = AppClient::new().await;
    let payments = PaymentsClient::new(app_client.clone(), None).await;

    let admin = payments.querier.admin().await.unwrap().unwrap();

    shared_tests::payments::get_admin(&payments.querier, &admin).await;
}

#[tokio::test]
async fn register_receives_open_account() {
    tracing_init();

    let app_client = AppClient::new().await;
    let payments = PaymentsClient::new(app_client.clone(), None).await;

    shared_tests::payments::register_recieves_open_account(
        &payments.querier,
        &payments.executor,
        RegisterReceivesOpenAccountProps {
            tg_handle: "@alice".to_string(),
            user_addr: app_client.rand_addr().await,
        },
    )
    .await;
}
