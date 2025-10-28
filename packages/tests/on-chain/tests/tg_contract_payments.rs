use cosmwasm_std::Addr;
use layer_climb::prelude::*;
use layer_climb_proto::Any;
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
            user_addr: app_client.rand_address().await.into(),
        },
    )
    .await;
}

// This is the minimal full flow of two registered accounts, one sending to the other.
// We should add more later with:
#[tokio::test]
async fn fund_account_and_send_workflow() {
    tracing_init();

    let app_client = AppClient::new().await;
    let payments = PaymentsClient::new(app_client.clone(), None).await;

    // Alice will send
    let tg_alice = "@alice";
    // Note: this also taps the facuet for 1_000_000_000 initial tokens
    let alice = app_client.rand_signing_client().await;
    let tg_bob = "@bob";
    let bob_addr = app_client.rand_address().await; // Bob just needs to watch

    // Query balances and assert alice (non-zero), bob (zero)
    let alice_balance = get_balance(&alice, None).await;
    let bob_balance = get_balance(&alice, Some(bob_addr.clone().into())).await;

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
    let grant = cosmwasm_std::coin(500_000u128, gas_denom);
    let msgs = build_registration_messages(
        &alice,
        tg_alice,
        &payments.querier.addr.clone().into(),
        grant,
    )
    .await;
    let _tx_resp = alice.tx_builder().broadcast(msgs).await.unwrap();

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
    let send_amount = 200_000u128;
    payments
        .executor
        .send_payment(tg_alice, tg_bob, send_amount, gas_denom)
        .await
        .unwrap();

    // Ensure the tokens were sent
    let alice_balance_new = get_balance(&alice, None).await;
    let bob_balance = get_balance(&alice, Some(bob_addr.into())).await;
    println!("Alice balance: {}", alice_balance);
    println!("Bob balance: {}", bob_balance);

    assert!(
        alice_balance >= alice_balance_new + send_amount,
        "alice's balance should have gone down by at least the amount sent"
    );
    assert_eq!(
        bob_balance, send_amount,
        "bob should have gotten the sent amount"
    );
}

// In this test, alice sends tokens to bob, who has not yet registered to receive payments.
// It works, and when bob later registers, he immediately receives the pending payment.
#[tokio::test]
async fn send_payment_then_register_receiver() {
    tracing_init();

    let app_client = AppClient::new().await;
    let payments = PaymentsClient::new(app_client.clone(), None).await;

    // Alice will send
    let tg_alice = "@alice";
    let alice = app_client.rand_signing_client().await;
    // Bob will receive
    let tg_bob = "@bob";
    let bob_addr = app_client.rand_address().await; // Bob just needs to watch

    // WAVS Admin registers Alice to receive payments
    payments
        .executor
        .register_receive(tg_alice.to_string(), &alice.addr.clone().into())
        .await
        .unwrap();

    // Alice registers to send funds and gives grant message in one tx
    let gas_denom = &alice.querier.chain_config.gas_denom;
    let grant = cosmwasm_std::coin(500_000u128, gas_denom);
    let msgs = build_registration_messages(
        &alice,
        tg_alice,
        &payments.querier.addr.clone().into(),
        grant,
    )
    .await;
    let _tx_resp = alice.tx_builder().broadcast(msgs).await.unwrap();
    let alice_balance = get_balance(&alice, None).await;

    // WAVS Admin triggers send from alice to bob
    let send_amount = 200_000u128;
    payments
        .executor
        .send_payment(tg_alice, tg_bob, send_amount, gas_denom)
        .await
        .unwrap();

    // Ensure the tokens were held in escrow but not received
    let alice_balance_new = get_balance(&alice, None).await;
    assert!(
        alice_balance >= alice_balance_new + send_amount,
        "alice's balance should have gone down by at least the amount sent"
    );
    let bob_balance = get_balance(&alice, Some(bob_addr.clone().into())).await;
    assert_eq!(bob_balance, 0u128, "bob should have gotten the sent amount");

    // WAVS Admin registers Bob to receive payments
    payments
        .executor
        .register_receive(tg_bob.to_string(), &bob_addr)
        .await
        .unwrap();

    // Now bob got paid
    let bob_balance = get_balance(&alice, Some(bob_addr.clone().into())).await;
    assert_eq!(
        bob_balance, send_amount,
        "bob should have gotten the sent amount"
    );
}

// In this test, both alice and bob register to receive payments.
// Alice then registers to send with a grant of 500_000 tokens.
// Alice sends 200_000 tokens to bob, which works (check balances)
// Alice then sends another 250_000 tokens that also work (multiple sends, but below the grant)
// Finally, Alice tries to send a final 100_000 tokens, but this fails as it hits the grant limit. 
// Note that we assert the final error message string, so we can use that to detect when a fill up is needed. 
#[tokio::test]
async fn send_multiple_payments() {
    tracing_init();

    let app_client = AppClient::new().await;
    let payments = PaymentsClient::new(app_client.clone(), None).await;

    // Alice will send
    let tg_alice = "@alice";
    let alice = app_client.rand_signing_client().await;
    // Bob will receive
    let tg_bob = "@bob";
    let bob_addr = app_client.rand_address().await;

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
    let grant = cosmwasm_std::coin(500_000u128, gas_denom);
    let msgs = build_registration_messages(
        &alice,
        tg_alice,
        &payments.querier.addr.clone().into(),
        grant,
    )
    .await;
    let _tx_resp = alice.tx_builder().broadcast(msgs).await.unwrap();

    let alice_balance = get_balance(&alice, None).await;
    let bob_balance = get_balance(&alice, Some(bob_addr.clone().into())).await;

    // First send: 200_000 tokens (should succeed)
    let send_amount_1 = 200_000u128;
    payments
        .executor
        .send_payment(tg_alice, tg_bob, send_amount_1, gas_denom)
        .await
        .unwrap();

    // Verify first send worked
    let alice_balance_after_1 = get_balance(&alice, None).await;
    let bob_balance_after_1 = get_balance(&alice, Some(bob_addr.clone().into())).await;
    assert!(
        alice_balance >= alice_balance_after_1 + send_amount_1,
        "alice's balance should have gone down by at least the first amount sent"
    );
    assert_eq!(
        bob_balance_after_1,
        bob_balance + send_amount_1,
        "bob should have received the first payment"
    );

    // Second send: 250_000 tokens (should succeed, total 450_000 < 500_000 grant)
    let send_amount_2 = 250_000u128;
    payments
        .executor
        .send_payment(tg_alice, tg_bob, send_amount_2, gas_denom)
        .await
        .unwrap();

    // Verify second send worked
    let alice_balance_after_2 = get_balance(&alice, None).await;
    let bob_balance_after_2 = get_balance(&alice, Some(bob_addr.clone().into())).await;
    assert!(
        alice_balance_after_1 >= alice_balance_after_2 + send_amount_2,
        "alice's balance should have gone down by at least the second amount sent"
    );
    assert_eq!(
        bob_balance_after_2,
        bob_balance_after_1 + send_amount_2,
        "bob should have received the second payment"
    );

    // Third send: 100_000 tokens (should fail, total would be 550_000 > 500_000 grant)
    let send_amount_3 = 100_000u128;
    let result = payments
        .executor
        .send_payment(tg_alice, tg_bob, send_amount_3, gas_denom)
        .await;

    // Assert that the third send failed
    assert!(result.is_err(), "third send should fail due to grant limit");
    
    // Check the error message to detect when a fill up is needed
    // Note that it is only visible in the "Caused by" section of the anyhow error
    let full_error = format!("{:?}", result.unwrap_err());
    println!("Error message: {:?}", full_error);
    assert!(full_error.contains("requested amount is more than spend limit"));

    // Verify balances didn't change after failed send
    let alice_balance_final = get_balance(&alice, None).await;
    let bob_balance_final = get_balance(&alice, Some(bob_addr.clone().into())).await;
    assert_eq!(
        alice_balance_final, alice_balance_after_2,
        "alice's balance should not change after failed send"
    );
    assert_eq!(
        bob_balance_final, bob_balance_after_2,
        "bob's balance should not change after failed send"
    );
}

async fn get_balance(client: &SigningClient, addr: Option<Address>) -> u128 {
    let addr = addr.unwrap_or_else(|| client.addr.clone());
    client
        .querier
        .balance(addr, None)
        .await
        .unwrap()
        .unwrap_or_default()
}

/// This builds messages for a user to register and grant permission to send on their behalf.
/// It must be signed by the users private key and then submitted as a multi-msg tx
async fn build_registration_messages(
    granter: &SigningClient,
    tg_handle: &str,
    contract_addr: &Addr,
    grant_amount: cosmwasm_std::Coin,
) -> Vec<Any> {
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

    vec![
        proto_into_any(&exec_msg).unwrap(),
        proto_into_any(&grant_msg).unwrap(),
    ]
}
