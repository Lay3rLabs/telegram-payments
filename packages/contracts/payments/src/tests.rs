use cosmwasm_std::{
    testing::{mock_dependencies, mock_env},
    Addr, MessageInfo,
};
use cw_multi_test::{App, ContractWrapper, Executor};
use tg_contract_api::payments::msg::{
    Auth, ChainAddrResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RegisterReceiveMsg,
    TgHandleResponse,
};

use crate::{execute, instantiate, query};

/// Helper function to instantiate the contract with admin auth
fn setup_contract_with_admin(app: &mut App, admin: &Addr) -> Addr {
    let code = ContractWrapper::new_with_empty(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let msg = InstantiateMsg {
        allowed_denoms: vec!["untrn".to_string(), "uatom".to_string()],
        auth: Auth::Admin(admin.to_string()),
    };

    let result =
        app.instantiate_contract(code_id, admin.clone(), &msg, &[], "telegram-payments", None);

    match &result {
        Ok(addr) => println!("Contract instantiated at: {}", addr),
        Err(e) => println!("Instantiation error: {:?}", e),
    }

    result.unwrap()
}

#[test]
fn test_instantiate_unit() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let sender = deps.api.addr_make("creator");
    let admin = deps.api.addr_make("admin");

    let info = MessageInfo {
        sender,
        funds: vec![],
    };

    let msg = InstantiateMsg {
        allowed_denoms: vec!["untrn".to_string(), "uatom".to_string()],
        auth: Auth::Admin(admin.to_string()),
    };

    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.attributes.len(), 2);
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "instantiate");
}

#[test]
fn test_instantiate_with_admin() {
    let mut app = App::default();
    let admin = app.api().addr_make("admin");

    // Debug: Test serialization/deserialization
    let msg = InstantiateMsg {
        allowed_denoms: vec!["untrn".to_string(), "uatom".to_string()],
        auth: Auth::Admin(admin.to_string()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    println!("InstantiateMsg JSON: {}", json);

    // Try to deserialize it back
    let deserialized: Result<InstantiateMsg, _> = serde_json::from_str(&json);
    println!("Deserialization result: {:?}", deserialized);

    // Try with cosmwasm_std serialization
    let binary = cosmwasm_std::to_json_vec(&msg).unwrap();
    let from_binary: Result<InstantiateMsg, _> = cosmwasm_std::from_json(&binary);
    println!("CosmWasm deserialization result: {:?}", from_binary);

    let contract_addr = setup_contract_with_admin(&mut app, &admin);

    // Query admin to verify it was set correctly
    let admin_response: tg_contract_api::payments::msg::AdminResponse = app
        .wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Admin {})
        .unwrap();

    assert_eq!(admin_response.admin, Some(admin.to_string()));
}

#[test]
fn test_register_receive_creates_open_account() {
    let mut app = App::default();
    let admin = app.api().addr_make("admin");

    let user_addr = app.api().addr_make("user123");
    let tg_handle = "@alice".to_string();

    let contract_addr = setup_contract_with_admin(&mut app, &admin);

    // Register user to receive payments
    let register_msg = ExecuteMsg::RegisterReceive(RegisterReceiveMsg {
        tg_handle: tg_handle.clone(),
        chain_addr: user_addr.to_string(),
    });

    app.execute_contract(admin.clone(), contract_addr.clone(), &register_msg, &[])
        .unwrap();

    // Query by Telegram handle - should return the registered address
    let addr_response: ChainAddrResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::AddrByTg {
                handle: tg_handle.clone(),
            },
        )
        .unwrap();

    assert_eq!(addr_response.addr, Some(user_addr.to_string()));

    // Query by address - should return None because this is only an OPEN account, not FUNDED
    let tg_response: TgHandleResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::TgByAddr {
                account: user_addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        tg_response.handle, None,
        "User should not be in FUNDED_ACCOUNTS yet"
    );
}

#[test]
fn test_register_receive_prevents_duplicate_tg_handle() {
    let mut app = App::default();
    let admin = app.api().addr_make("admin");

    let user1_addr = app.api().addr_make("user1");
    let user2_addr = app.api().addr_make("user2");
    let tg_handle = "@alice".to_string();

    let contract_addr = setup_contract_with_admin(&mut app, &admin);

    // Register first user
    let register_msg1 = ExecuteMsg::RegisterReceive(RegisterReceiveMsg {
        tg_handle: tg_handle.clone(),
        chain_addr: user1_addr.to_string(),
    });

    app.execute_contract(admin.clone(), contract_addr.clone(), &register_msg1, &[])
        .unwrap();

    // Try to register second user with same Telegram handle - should fail
    let register_msg2 = ExecuteMsg::RegisterReceive(RegisterReceiveMsg {
        tg_handle: tg_handle.clone(),
        chain_addr: user2_addr.to_string(),
    });

    let err = app
        .execute_contract(admin, contract_addr, &register_msg2, &[])
        .unwrap_err();

    assert!(
        err.to_string()
            .contains(&format!("TG Handle {} is already registered", tg_handle)),
        "Expected TgAlreadyRegistered error, got: {}",
        err
    );
}

#[test]
fn test_register_receive_requires_admin() {
    let mut app = App::default();
    let admin = app.api().addr_make("admin");

    let unauthorized = app.api().addr_make("unauthorized");
    let user_addr = app.api().addr_make("user123");
    let tg_handle = "@alice".to_string();

    let contract_addr = setup_contract_with_admin(&mut app, &admin);

    // Try to register from unauthorized address
    let register_msg = ExecuteMsg::RegisterReceive(RegisterReceiveMsg {
        tg_handle: tg_handle.clone(),
        chain_addr: user_addr.to_string(),
    });

    let err = app
        .execute_contract(unauthorized, contract_addr, &register_msg, &[])
        .unwrap_err();

    assert!(
        err.to_string().contains("Unauthorized"),
        "Expected Unauthorized error, got: {}",
        err
    );
}

#[test]
fn test_query_nonexistent_accounts() {
    let mut app = App::default();
    let admin = app.api().addr_make("admin");

    let contract_addr = setup_contract_with_admin(&mut app, &admin);

    // Query non-existent Telegram handle
    let addr_response: ChainAddrResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::AddrByTg {
                handle: "@nonexistent".to_string(),
            },
        )
        .unwrap();

    assert_eq!(addr_response.addr, None);

    let bad_addr = app.api().addr_make("nonexistent123");
    // Query non-existent address
    let tg_response: TgHandleResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::TgByAddr {
                account: bad_addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(tg_response.handle, None);
}

#[test]
fn test_allowed_denoms_query() {
    let mut app = App::default();
    let admin = app.api().addr_make("admin");

    let contract_addr = setup_contract_with_admin(&mut app, &admin);

    // Query allowed denoms
    let denoms: Vec<String> = app
        .wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::AllowedDenoms {})
        .unwrap();

    assert_eq!(denoms, vec!["untrn".to_string(), "uatom".to_string()]);
}
