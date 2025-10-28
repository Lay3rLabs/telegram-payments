mod command;
mod context;
mod ipfs;
mod output;

use std::process::exit;

use serde::{de::DeserializeOwned, Deserialize};
use tg_utils::{faucet, tracing::tracing_init};
use wavs_types::{
    ComponentSource, Service, ServiceManager, SignatureKind, Submit, Trigger, Workflow,
};

use crate::{
    command::{AuthKind, CliCommand, ContractKind},
    context::CliContext,
    ipfs::IpfsFile,
    output::{
        OutputComponentUpload, OutputContractInstantiate, OutputContractUpload, OutputServiceUpload,
    },
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Install rustls crypto provider before any TLS operations
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    tracing_init();

    let ctx = CliContext::new().await;

    match ctx.command.clone() {
        CliCommand::AssertAccountExists { addr, args: _ } => {
            let client = ctx.query_client().await.unwrap();
            let addr = match addr {
                Some(addr) => ctx.parse_address(&addr).await.unwrap(),
                None => ctx.wallet_addr().await.unwrap(),
            };
            let balance = client
                .balance(addr.clone(), None)
                .await
                .unwrap()
                .unwrap_or_default();

            if balance == 0 {
                eprintln!(
                    "{} has zero balance. Please fund the wallet before proceeding.",
                    addr
                );
                exit(1);
            }
        }
        CliCommand::UploadContract { kind, args } => {
            let client = ctx.signing_client().await.unwrap();

            let (code_id, tx_resp) = client
                .contract_upload_file(kind.wasm_bytes().await, None)
                .await
                .unwrap();

            println!("Uploaded {kind} contract with code ID: {code_id}");

            args.output()
                .write(OutputContractUpload {
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
                .write(OutputContractInstantiate {
                    kind: ContractKind::Payments,
                    address: contract_addr.to_string(),
                    tx_hash: tx_resp.txhash,
                })
                .await
                .unwrap();
        }
        CliCommand::FaucetTap {
            addr,
            amount,
            denom,
            args: _,
        } => {
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
            faucet::tap(&addr, amount, denom.as_deref()).await.unwrap();
            let balance_after = client
                .balance(addr.clone(), None)
                .await
                .unwrap()
                .unwrap_or_default();

            println!(
                "Tapped faucet for {addr} - balance before: {balance_before} balance after: {balance_after}"
            );
        }
        CliCommand::UploadComponent {
            kind,
            args,
            ipfs_api_url,
            ipfs_gateway_url,
        } => {
            let bytes = kind.wasm_bytes().await;

            let digest = wavs_types::ComponentDigest::hash(&bytes);

            let resp = IpfsFile::upload(
                bytes,
                &format!("{kind}.wasm"),
                ipfs_api_url.as_ref(),
                ipfs_gateway_url.as_ref(),
                true,
            )
            .await
            .unwrap();

            let IpfsFile {
                cid,
                uri,
                gateway_url,
            } = resp;

            args.output()
                .write(OutputComponentUpload {
                    kind,
                    digest,
                    cid,
                    uri,
                    gateway_url,
                })
                .await
                .unwrap();
        }
        CliCommand::UploadService {
            args,
            ipfs_api_url,
            ipfs_gateway_url,
            contract_payments_instantiation_file,
            component_operator_cid_file,
            component_aggregator_cid_file,
            cron_schedule,
            middleware_instantiation_file,
            aggregator_url,
        } => {
            let output_directory = args.output().directory;

            let contract_payments_instantiation_file =
                output_directory.join(contract_payments_instantiation_file);
            let component_operator_cid_file = output_directory.join(component_operator_cid_file);
            let component_aggregator_cid_file =
                output_directory.join(component_aggregator_cid_file);
            let middleware_instantiation_file =
                output_directory.join(middleware_instantiation_file);

            async fn read_and_decode<T: DeserializeOwned>(path: std::path::PathBuf) -> T {
                match tokio::fs::read_to_string(&path).await {
                    Err(e) => {
                        panic!("Failed to read file {}: {}", path.display(), e);
                    }
                    Ok(content) => match serde_json::from_str(&content) {
                        Err(e) => {
                            panic!("Failed to decode JSON from file {}: {}", path.display(), e);
                        }
                        Ok(data) => data,
                    },
                }
            }

            let contract_payments: OutputContractInstantiate =
                read_and_decode(contract_payments_instantiation_file).await;

            let component_operator: OutputComponentUpload =
                read_and_decode(component_operator_cid_file).await;

            let component_aggregator: OutputComponentUpload =
                read_and_decode(component_aggregator_cid_file).await;

            #[derive(Debug, Deserialize)]
            struct MiddlewareInstantiation {
                #[serde(rename = "registry_address")]
                pub _registry_address: String,
                pub service_manager_address: String,
            }

            let middleware_instantiation: MiddlewareInstantiation =
                read_and_decode(middleware_instantiation_file).await;

            let trigger = Trigger::Cron {
                schedule: cron_schedule,
                start_time: None,
                end_time: None,
            };

            let operator_component = wavs_types::Component {
                source: ComponentSource::Download {
                    //uri: component_operator.uri.parse().unwrap(),
                    uri: component_operator.gateway_url.parse().unwrap(),
                    digest: component_operator.digest,
                },
                permissions: wavs_types::Permissions {
                    allowed_http_hosts: wavs_types::AllowedHostPermission::All,
                    file_system: false,
                },
                fuel_limit: None,
                time_limit_seconds: None,
                config: Default::default(),
                env_keys: Default::default(),
            };

            let aggregator_component = wavs_types::Component {
                source: ComponentSource::Download {
                    //uri: component_aggregator.uri.parse().unwrap(),
                    uri: component_aggregator.gateway_url.parse().unwrap(),
                    digest: component_aggregator.digest,
                },
                permissions: wavs_types::Permissions {
                    allowed_http_hosts: wavs_types::AllowedHostPermission::All,
                    file_system: false,
                },
                fuel_limit: None,
                time_limit_seconds: None,
                config: [
                    (
                        "PAYMENTS_CONTRACT_ADDRESS".to_string(),
                        contract_payments.address,
                    ),
                    ("CHAIN".to_string(), args.chain.to_string()),
                ]
                .into_iter()
                .collect(),
                env_keys: Default::default(),
            };

            let submit = Submit::Aggregator {
                url: aggregator_url.to_string(),
                component: Box::new(aggregator_component),
                signature_kind: SignatureKind::evm_default(),
            };

            let workflow = Workflow {
                trigger,
                component: operator_component,
                submit,
            };

            let service = Service {
                name: "Telegram Payments".to_string(),
                workflows: [("workflow-1".parse().unwrap(), workflow)]
                    .into_iter()
                    .collect(),
                status: wavs_types::ServiceStatus::Active,
                manager: ServiceManager::Cosmos {
                    chain: args.chain.clone(),
                    address: middleware_instantiation
                        .service_manager_address
                        .parse()
                        .unwrap(),
                },
            };

            let bytes = serde_json::to_vec_pretty(&service).unwrap();

            let digest = wavs_types::ServiceDigest::hash(&bytes);

            let resp = IpfsFile::upload(
                bytes,
                "service.json",
                ipfs_api_url.as_ref(),
                ipfs_gateway_url.as_ref(),
                true,
            )
            .await
            .unwrap();

            let IpfsFile {
                cid,
                uri,
                gateway_url,
            } = resp;

            args.output()
                .write(OutputServiceUpload {
                    service,
                    digest,
                    cid,
                    uri: uri.clone(),
                    gateway_url: gateway_url.clone(),
                })
                .await
                .unwrap();

            println!("\nService URI: {}", uri);
            println!("Service Gateway URL: {}\n", gateway_url);
        }
        CliCommand::AggregatorRegisterService {
            args,
            service_manager_address,
            aggregator_url,
        } => {
            let req = wavs_types::aggregator::RegisterServiceRequest {
                service_manager: ServiceManager::Cosmos {
                    chain: args.chain,
                    address: service_manager_address.parse().unwrap(),
                },
            };

            reqwest::Client::new()
                .post(aggregator_url.join("services").unwrap())
                .json(&req)
                .send()
                .await
                .unwrap()
                .error_for_status()
                .unwrap();
        }

        CliCommand::OperatorAddService {
            args,
            service_manager_address,
            wavs_url,
        } => {
            let req = wavs_types::AddServiceRequest {
                service_manager: ServiceManager::Cosmos {
                    chain: args.chain,
                    address: service_manager_address.parse().unwrap(),
                },
            };

            reqwest::Client::new()
                .post(wavs_url.join("services").unwrap())
                .json(&req)
                .send()
                .await
                .unwrap()
                .error_for_status()
                .unwrap();
        }
    }
}
