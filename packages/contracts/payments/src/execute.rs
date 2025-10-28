use crate::state::{
    ADMIN, ALLOWED_DENOMS, FUNDED_ACCOUNTS, OPEN_ACCOUNTS, PENDING_PAYMENTS, SERVICE_MANAGER,
};
use cosmwasm_std::{ensure, AnyMsg, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, Uint256};
use layer_climb_proto::Any;
use layer_climb_proto::{authz::MsgExec, bank::MsgSend, Coin as ProtoCoin, Message, Name};
use tg_contract_api::payments::msg::{RegisterReceiveMsg, SendPaymentMsg, WavsPayload};
use wavs_types::contracts::cosmwasm::service_manager::ServiceManagerQueryMessages;
use wavs_types::contracts::cosmwasm::{
    service_handler::{WavsEnvelope, WavsSignatureData},
    service_manager::WavsValidateResult,
};

use crate::error::ContractError;

pub fn register_send(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    tg_handle: String,
) -> Result<Response, ContractError> {
    // Don't overwrite anything already registered
    let chain_addr = info.sender;
    if FUNDED_ACCOUNTS.has(deps.storage, &chain_addr) {
        return Err(ContractError::AddrAlreadyRegistered(chain_addr));
    }

    // Ensure this address matches the previous receive registration
    let registered_receive = OPEN_ACCOUNTS.load(deps.storage, &tg_handle)?;
    ensure!(
        registered_receive == chain_addr,
        ContractError::Unauthorized
    ); // TODO: better error message

    FUNDED_ACCOUNTS.save(deps.storage, &chain_addr, &tg_handle)?;

    Ok(Response::new()
        .add_attribute("method", "register_send")
        .add_attribute("tg_handle", tg_handle)
        .add_attribute("chain_addr", chain_addr))
}

pub fn register_receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: RegisterReceiveMsg,
) -> Result<Response, ContractError> {
    // TODO: better error messages
    let admin = ADMIN.load(deps.storage)?;
    ensure!(info.sender == admin, ContractError::Unauthorized);

    _register_receive(deps, msg.tg_handle, msg.chain_addr)
}

pub fn send_payment(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: SendPaymentMsg,
) -> Result<Response, ContractError> {
    // TODO: better error messages
    let admin = ADMIN.load(deps.storage)?;
    ensure!(info.sender == admin, ContractError::Unauthorized);

    _send_payment(deps, _env, msg.from_tg, msg.to_tg, msg.amount, msg.denom)
}

pub fn wavs_handle_envelope(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    envelope: WavsEnvelope,
    signature_data: WavsSignatureData,
) -> Result<Response, ContractError> {
    let service_manager = SERVICE_MANAGER.load(deps.storage)?;

    deps.querier.query_wasm_smart::<WavsValidateResult>(
        service_manager,
        &ServiceManagerQueryMessages::WavsValidate {
            envelope: envelope.clone(),
            signature_data: signature_data.clone(),
        },
    )?;

    let payload: WavsPayload = cosmwasm_std::from_json(envelope.as_slice())?;
    match payload {
        WavsPayload::Register(msg) => _register_receive(deps, msg.tg_handle, msg.chain_addr),
        WavsPayload::SendPayment(msg) => {
            _send_payment(deps, _env, msg.from_tg, msg.to_tg, msg.amount, msg.denom)
        }
    }
}

pub fn _register_receive(
    deps: DepsMut,
    tg_handle: String,
    chain_addr: String,
) -> Result<Response, ContractError> {
    // Don't overwrite anything already registered
    let chain_addr = deps.api.addr_validate(&chain_addr)?;
    if OPEN_ACCOUNTS.has(deps.storage, &tg_handle) {
        return Err(ContractError::TgAlreadyRegistered(tg_handle));
    }
    OPEN_ACCOUNTS.save(deps.storage, &tg_handle, &chain_addr)?;

    let mut resp = Response::new().add_attribute("method", "register_receive");

    // TODO: check if there are any pending payments for this tg_handle and send them
    if let Some(pending) = PENDING_PAYMENTS.may_load(deps.storage, &tg_handle)? {
        PENDING_PAYMENTS.remove(deps.storage, &tg_handle);
        let msg = BankMsg::Send {
            to_address: chain_addr.to_string(),
            amount: pending.balance(),
        };
        resp = resp.add_message(msg);
    }

    Ok(resp
        .add_attribute("tg_handle", tg_handle)
        .add_attribute("chain_addr", chain_addr))
}

pub fn _send_payment(
    deps: DepsMut,
    env: Env,
    from_tg: String,
    to_tg: String,
    amount: Uint256,
    denom: String,
) -> Result<Response, ContractError> {
    // Check it is an allowed denom
    let allowed_denoms = ALLOWED_DENOMS.load(deps.storage)?;
    ensure!(
        allowed_denoms.contains(&denom),
        ContractError::TokenNotWhitelisted { token: denom }
    );
    // Ensure amount > 0
    ensure!(amount > Uint256::zero(), ContractError::ZeroSend);
    let amount = Coin { amount, denom };

    // Ensure address this account is sending from
    // FIXME: better error messages, not NotFound
    // need to reverse lookup this to ensure it is proper
    let from_addr = OPEN_ACCOUNTS.load(deps.storage, &from_tg)?;
    let check_from = FUNDED_ACCOUNTS.load(deps.storage, &from_addr)?;
    ensure!(check_from == from_tg, ContractError::Unauthorized);

    // Figure out where to send it to
    let to_addr = match OPEN_ACCOUNTS.may_load(deps.storage, &to_tg)? {
        Some(addr) => addr,
        None => {
            // Record the pending payment
            let mut pending = PENDING_PAYMENTS
                .may_load(deps.storage, &to_tg)?
                .unwrap_or_default();
            pending.add_payment(amount.clone());
            PENDING_PAYMENTS.save(deps.storage, &to_tg, &pending)?;

            // Send to this contract
            env.contract.address.clone()
        }
    };

    // Custom bank MsgSend from the original sender, not the contract
    let msg_send = MsgSend {
        from_address: from_addr.to_string(),
        to_address: to_addr.to_string(),
        amount: vec![ProtoCoin {
            amount: amount.amount.to_string(),
            denom: amount.denom.clone(),
        }],
    };
    // Wrapped in an authz MsgExec so it uses our allowances to send
    let msg_exec = MsgExec {
        grantee: env.contract.address.to_string(),
        msgs: vec![Any {
            type_url: MsgSend::type_url(),
            value: msg_send.encode_to_vec().into(),
        }],
    };
    // Converted into opaque protobut AnyMsg for wasmd to handle
    let any_msg = AnyMsg {
        type_url: MsgExec::type_url(),
        value: msg_exec.encode_to_vec().into(),
    };

    Ok(Response::new()
        .add_message(any_msg)
        .add_attribute("method", "send_payment")
        .add_attribute("from_tg", from_tg)
        .add_attribute("to_tg", to_tg)
        .add_attribute("amount", amount.amount)
        .add_attribute("denom", amount.denom))
}
