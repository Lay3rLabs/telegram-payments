use off_chain_tests::client::{payments::PaymentsClient, AppClient};
use tg_utils::tracing::tracing_init;

#[tokio::test]
async fn payments_works() {
    tracing_init();

    let app_client = AppClient::new("admin");
    let payments_client = PaymentsClient::new(app_client.clone());
}
