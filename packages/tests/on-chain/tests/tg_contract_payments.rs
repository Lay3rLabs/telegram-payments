use cosmwasm_std::Addr;
use layer_climb::prelude::*;
use layer_climb::proto::Coin as ProtoCoin;
use on_chain_tests::client::{payments::PaymentsClient, AppClient};
use tg_contract_api::payments::msg::ExecuteMsg;
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

// This is the minimal full flow of two registered accounts, one sending to the other.
// We should add more later with:
// * pending payments (bob registers later)
// * multiples sends from alice (under total limit)
// * multiples sends from alice (over total limit)
#[tokio::test]
async fn fund_account_and_send_workflow() {
    tracing_init();

    let app_client = AppClient::new().await;
    let payments = PaymentsClient::new(app_client.clone(), None).await;

    // Alice will send
    let tg_alice = "@alice";
    let alice_addr = app_client.rand_addr().await; // TODO: we need a real key here to sign with
    let tg_bob = "@bob";
    let bob_addr = app_client.rand_addr().await; // Bob just needs to watch

    // TODO: Give some tokens to Alice

    // TODO: Query balance of alice (non-zero), bob (zero)

    // WAVS Admin registers Alice to receive payments
    shared_tests::payments::register_recieves_open_account(
        &payments.querier,
        &payments.executor,
        RegisterReceivesOpenAccountProps {
            tg_handle: tg_alice.to_string(),
            user_addr: alice_addr.clone(),
        },
    )
    .await;

    // WAVS Admin registers Bob to receive payments
    shared_tests::payments::register_recieves_open_account(
        &payments.querier,
        &payments.executor,
        RegisterReceivesOpenAccountProps {
            tg_handle: tg_bob.to_string(),
            user_addr: bob_addr.clone(),
        },
    )
    .await;

    // Alice registers to send funds and gives grant message in one tx
    let grant = cosmwasm_std::coin(2_000_000_000u128, "untrn");
    let (msg1, msg2) = build_registration_messages(
        &app_client,
        tg_alice,
        &alice_addr,
        &payments.querier.addr,
        grant,
    )
    .await;

    app_client
        .pool()
        .get()
        .await
        .unwrap()
        .tx_builder()
        .broadcast([
            proto_into_any(&msg1).unwrap(),
            proto_into_any(&msg2).unwrap(),
        ])
        .await
        .unwrap();

    // Query alice reverse mapping is now set
    assert_eq!(
        payments
            .querier
            .tg_handle_by_addr(tg_alice.to_string())
            .await
            .unwrap(),
        Some(alice_addr.to_string())
    );

    // WAVS Admin triggers send from alice to bob
    payments
        .executor
        .send_payment(tg_alice, tg_bob, 1_000_000_000u128, "untrn")
        .await
        .unwrap();

    // TODO: Query balances of Alice and Bob updated
}

/// This builds messages for a user to register and grant permission to send on their behalf.
/// It must be signed by the users private key and then submitted as a multi-msg tx
async fn build_registration_messages(
    app_client: &AppClient,
    tg_handle: &str,
    user_addr: &Addr,
    contract_addr: &Addr,
    grant_amount: cosmwasm_std::Coin,
) -> (
    layer_climb_proto::wasm::MsgExecuteContract,
    layer_climb_proto::authz::MsgGrant,
) {
    let signing_client = app_client.pool().get().await.unwrap();

    let user_addr: Address = CosmosAddr::try_from(user_addr).unwrap().into();
    let contract_addr: Address = CosmosAddr::try_from(contract_addr).unwrap().into();

    let register_msg = ExecuteMsg::RegisterSend {
        tg_handle: tg_handle.to_string(),
    };

    let exec_msg = signing_client
        .contract_execute_msg(&contract_addr, vec![], &register_msg)
        .unwrap();

    let grant_msg = signing_client
        .authz_grant_send_msg(
            user_addr,
            contract_addr,
            vec![ProtoCoin {
                denom: grant_amount.denom,
                amount: grant_amount.amount.to_string(),
            }],
            vec![],
        )
        .unwrap();

    return (exec_msg, grant_msg);
}
