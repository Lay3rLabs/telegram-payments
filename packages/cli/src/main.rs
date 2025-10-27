mod command;
mod context;
mod output;

use tg_utils::{faucet, tracing::tracing_init};

use crate::{
    command::{AuthKind, CliCommand, ContractKind},
    context::CliContext,
    output::OutputData,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Install rustls crypto provider before any TLS operations
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    tracing_init();

    let ctx = CliContext::new().await;

    match ctx.command.clone() {
        CliCommand::UploadContract { kind, args } => {
            let client = ctx.signing_client().await.unwrap();

            let (code_id, tx_resp) = client
                .contract_upload_file(kind.wasm_bytes().await, None)
                .await
                .unwrap();

            println!("Uploaded {kind} contract with code ID: {code_id}");

            args.output()
                .write(OutputData::ContractUpload {
                    kind,
                    code_id,
                    tx_hash: tx_resp.txhash,
                })
                .await
                .unwrap();
        }
        CliCommand::InstantiatePayments {
            allowed_denoms,
            auth,
            auth_kind,
            args,
            code_id,
        } => {
            let client = ctx.signing_client().await.unwrap();

            let auth = match auth_kind {
                AuthKind::ServiceManager => {
                    tg_contract_api::payments::msg::Auth::ServiceManager(match auth {
                        Some(addr) => ctx.parse_address(&addr).await.unwrap().to_string(),
                        None => {
                            panic!("Service manager auth requires an address to be provided")
                        }
                    })
                }
                AuthKind::User => tg_contract_api::payments::msg::Auth::Admin(match auth {
                    Some(addr) => ctx.parse_address(&addr).await.unwrap().to_string(),
                    None => ctx.wallet_addr().await.unwrap().to_string(),
                }),
            };

            let instantiate_msg = tg_contract_api::payments::msg::InstantiateMsg {
                allowed_denoms,
                auth,
            };

            let (contract_addr, tx_resp) = client
                .contract_instantiate(
                    None,
                    code_id,
                    "Telegram Payments",
                    &instantiate_msg,
                    vec![],
                    None,
                )
                .await
                .unwrap();

            println!("Instantiated Payments contract at address: {contract_addr}");

            args.output()
                .write(OutputData::ContractInstantiate {
                    kind: ContractKind::Payments,
                    address: contract_addr.to_string(),
                    tx_hash: tx_resp.txhash,
                })
                .await
                .unwrap();
        }
        CliCommand::FaucetTap { addr, url, args: _ } => {
            let client = ctx.query_client().await.unwrap();
            let addr = match addr {
                Some(addr) => ctx.parse_address(&addr).await.unwrap(),
                None => ctx.wallet_addr().await.unwrap(),
            };
            let balance_before = client
                .balance(addr.clone(), None)
                .await
                .unwrap()
                .unwrap_or_default();
            faucet::tap(&addr, &client.chain_config.gas_denom, Some(&url))
                .await
                .unwrap();
            let balance_after = client
                .balance(addr.clone(), None)
                .await
                .unwrap()
                .unwrap_or_default();

            println!(
                "Tapped faucet for {addr} - balance before: {balance_before} balance after: {balance_after}"
            );
        }
    }
}
