use tg_utils::{
    addr::AnyAddr,
    client::payments::{PaymentsExecutor, PaymentsQuerier},
};

pub async fn get_admin(querier: &PaymentsQuerier, expected: &str) {
    let admin = querier.admin().await.unwrap().unwrap();
    assert_eq!(admin, expected);
}

pub struct RegisterReceivesOpenAccountProps {
    pub user_addr: AnyAddr,
    pub tg_handle: String,
}

pub async fn register_recieves_open_account(
    querier: &PaymentsQuerier,
    executor: &PaymentsExecutor,
    props: RegisterReceivesOpenAccountProps,
) {
    let RegisterReceivesOpenAccountProps {
        user_addr,
        tg_handle,
    } = props;
    // Register user to receive payments
    executor
        .register_receive(tg_handle.clone(), &user_addr)
        .await
        .unwrap();

    // Query by Telegram handle - should return the registered address
    assert_eq!(
        querier.addr_by_tg_handle(tg_handle).await.unwrap().unwrap(),
        user_addr.to_string()
    );

    // Query by address - should return None because this is only an OPEN account, not FUNDED

    assert_eq!(
        querier
            .tg_handle_by_addr(user_addr.to_string())
            .await
            .unwrap(),
        None,
        "User should not be in FUNDED_ACCOUNTS yet"
    );
}
