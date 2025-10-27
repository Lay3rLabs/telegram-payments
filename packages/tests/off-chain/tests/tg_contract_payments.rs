use off_chain_tests::client::{payments::PaymentsClient, AppClient};
use tg_utils::tracing::tracing_init;

#[tokio::test]
async fn get_admin() {
    tracing_init();

    let app_client = AppClient::new("admin");
    let payments = PaymentsClient::new(app_client.clone(), None);

    let admin = payments.querier.admin().await.unwrap().unwrap();

    assert_eq!(admin, app_client.admin().to_string());
}

#[tokio::test]
async fn register_receive_creates_open_account() {
    tracing_init();

    let app_client = AppClient::new("admin");
    let payments = PaymentsClient::new(app_client.clone(), None);

    let user_addr = app_client.with_app(|app| app.api().addr_make("user123"));
    let tg_handle = "@alice".to_string();

    // Register user to receive payments
    payments
        .executor
        .register_receive(tg_handle.clone(), user_addr.clone())
        .await
        .unwrap();

    // Query by Telegram handle - should return the registered address
    assert_eq!(
        payments
            .querier
            .addr_by_tg_handle(tg_handle)
            .await
            .unwrap()
            .unwrap(),
        user_addr.to_string()
    );

    // Query by address - should return None because this is only an OPEN account, not FUNDED

    assert_eq!(
        payments
            .querier
            .tg_handle_by_addr(user_addr.to_string())
            .await
            .unwrap(),
        None,
        "User should not be in FUNDED_ACCOUNTS yet"
    );
}

#[tokio::test]
async fn register_receive_prevents_duplicate_tg_handle() {
    tracing_init();

    let app_client = AppClient::new("admin");
    let payments = PaymentsClient::new(app_client.clone(), None);

    let user1_addr = app_client.with_app(|app| app.api().addr_make("user1"));
    let user2_addr = app_client.with_app(|app| app.api().addr_make("user2"));

    let tg_handle = "@alice".to_string();

    // Register first user
    payments
        .executor
        .register_receive(tg_handle.clone(), user1_addr.clone())
        .await
        .unwrap();

    // Try to register second user with same Telegram handle - should fail
    let err = payments
        .executor
        .register_receive(tg_handle.clone(), user2_addr.clone())
        .await
        .unwrap_err();

    assert!(
        err.to_string()
            .contains(&format!("TG Handle {} is already registered", tg_handle)),
        "Expected TgAlreadyRegistered error, got: {}",
        err
    );
}

#[tokio::test]
async fn register_receive_requires_admin() {
    tracing_init();

    let app_client = AppClient::new("admin");
    let unauthorized = app_client.with_app(|app| app.api().addr_make("unauthorized"));

    let payments = PaymentsClient::new(app_client.clone(), Some(unauthorized));

    let user_addr = app_client.with_app(|app| app.api().addr_make("user123"));

    let tg_handle = "@alice".to_string();

    // Try to register user with unauthorized client - should fail
    let err = payments
        .executor
        .register_receive(tg_handle.clone(), user_addr.clone())
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("Unauthorized"),
        "Expected Unauthorized error, got: {}",
        err
    );
}

#[tokio::test]
async fn query_nonexistent_accounts() {
    tracing_init();

    let app_client = AppClient::new("admin");
    let payments = PaymentsClient::new(app_client.clone(), None);

    let tg_handle = "@alice".to_string();

    // Query non-existent Telegram handle
    assert_eq!(
        payments
            .querier
            .addr_by_tg_handle(tg_handle.clone())
            .await
            .unwrap(),
        None,
        "Expected no address for unregistered tg_handle"
    );

    // Query non-existent address
    let bad_addr = app_client.with_app(|app| app.api().addr_make("nonexistent123"));

    assert_eq!(
        payments
            .querier
            .tg_handle_by_addr(bad_addr.to_string())
            .await
            .unwrap(),
        None,
        "Expected no address for unregistered tg_handle"
    );
}

#[tokio::test]
async fn allowed_denoms() {
    tracing_init();

    let app_client = AppClient::new("admin");
    let payments = PaymentsClient::new(app_client.clone(), None);

    let mut allowed_denoms = payments.querier.allowed_denoms().await.unwrap();
    allowed_denoms.sort();

    assert_eq!(
        allowed_denoms,
        vec!["uatom".to_string(), "untrn".to_string()]
    );
}
