use cosmwasm_std::Addr;
use layer_climb::prelude::*;
use on_chain_tests::client::{payments::PaymentsClient, AppClient};
use tg_contract_api::payments::msg::ExecuteMsg;
use tg_test_common::shared_tests::{self, payments::RegisterReceivesOpenAccountProps};
use tg_utils::{faucet, tracing::tracing_init};

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
            user_addr: app_client.rand_address().await.into(),
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
    let alice = app_client.rand_signing_client().await;
    let tg_bob = "@bob";
    let bob_addr = app_client.rand_address().await; // Bob just needs to watch

    println!("Alice: {}", &alice.addr);
    println!("Bob: {}", &bob_addr);
    println!("Contract: {}", &payments.executor.addr);

    // Give some tokens to Alice
    faucet::tap(&alice.addr, None, None).await.unwrap();

    // Query balances and assert alice (non-zero), bob (zero)
    let alice_balance = alice
        .querier
        .balance(alice.addr.clone(), None)
        .await
        .unwrap()
        .unwrap_or_default();
    let bob_balance = alice
        .querier
        .balance(bob_addr.clone().into(), None)
        .await
        .unwrap()
        .unwrap_or_default();

    assert_ne!(
        alice_balance, 0u128,
        "alice should have been funded by faucet"
    );
    assert_eq!(bob_balance, 0u128, "bob should start with zero balance");

    // WAVS Admin registers Alice to receive payments
    payments
        .executor
        .register_receive(tg_alice.to_string(), &alice.addr.clone().into())
        .await
        .unwrap();

    // WAVS Admin registers Bob to receive payments
    payments
        .executor
        .register_receive(tg_bob.to_string(), &bob_addr)
        .await
        .unwrap();

    // Alice registers to send funds and gives grant message in one tx
    let gas_denom = &alice.querier.chain_config.gas_denom;
    let grant = cosmwasm_std::coin(500_000_000u128, gas_denom);

    let (msg1, msg2) = build_registration_messages(
        &alice,
        tg_alice,
        &payments.querier.addr.clone().into(),
        grant,
    )
    .await;

    let tx_resp = alice
        .tx_builder()
        .broadcast([
            proto_into_any(&msg1).unwrap(),
            proto_into_any(&msg2).unwrap(),
        ])
        .await
        .unwrap();

    for event in
        CosmosTxEvents::from(&tx_resp).filter_events_by_type("cosmos.authz.v1beta1.EventGrant")
    {
        println!("{:#?}", event);
    }

    // Query alice bidirectional mapping is now set
    assert_eq!(
        payments
            .querier
            .addr_by_tg_handle(tg_alice.to_string())
            .await
            .unwrap(),
        Some(alice.addr.to_string())
    );
    assert_eq!(
        payments
            .querier
            .tg_handle_by_addr(alice.addr.to_string())
            .await
            .unwrap(),
        Some(tg_alice.to_string())
    );

    // WAVS Admin triggers send from alice to bob
    payments
        .executor
        .send_payment(tg_alice, tg_bob, 200_000_000u128, "untrn")
        .await
        .unwrap();

    let alice_balance_new = alice
        .querier
        .balance(alice.addr.clone(), None)
        .await
        .unwrap()
        .unwrap_or_default();
    let bob_balance = alice
        .querier
        .balance(bob_addr.into(), None)
        .await
        .unwrap()
        .unwrap_or_default();

    assert_ne!(
        alice_balance, alice_balance_new,
        "alice's balance should have changed"
    );
    assert_ne!(bob_balance, 0u128, "bob should no longer have zero balance");
}

/// This builds messages for a user to register and grant permission to send on their behalf.
/// It must be signed by the users private key and then submitted as a multi-msg tx
async fn build_registration_messages(
    granter: &SigningClient,
    tg_handle: &str,
    contract_addr: &Addr,
    grant_amount: cosmwasm_std::Coin,
) -> (
    layer_climb_proto::wasm::MsgExecuteContract,
    layer_climb_proto::authz::MsgGrant,
) {
    let contract_addr: Address = CosmosAddr::try_from(contract_addr).unwrap().into();

    let register_msg = ExecuteMsg::RegisterSend {
        tg_handle: tg_handle.to_string(),
    };

    let exec_msg = granter
        .contract_execute_msg(&contract_addr, vec![], &register_msg)
        .unwrap();

    let grant_msg = granter
        .authz_grant_send_msg(
            None,
            contract_addr,
            vec![grant_amount.to_proto_coin()],
            vec![],
        )
        .unwrap();

    return (exec_msg, grant_msg);
}
