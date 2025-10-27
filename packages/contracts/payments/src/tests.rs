use cosmwasm_std::{
    testing::{mock_dependencies, mock_env},
    MessageInfo,
};
use tg_contract_api::payments::msg::{Auth, InstantiateMsg};

use crate::instantiate;

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
