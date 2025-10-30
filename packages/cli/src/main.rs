mod command;
mod context;
mod ipfs;
mod output;

use std::process::exit;

use cosmwasm_std::Uint256;
use layer_climb::prelude::EvmAddr;
use reqwest::Url;
use serde::{de::DeserializeOwned, Deserialize};
use tg_utils::{
    faucet, telegram::messenger::any_client::TelegramMessengerExt, tracing::tracing_init,
};
use wavs_types::{
    ComponentSource, GetSignerRequest, Service, ServiceManager, SignatureKind, SignerResponse,
    Submit, Trigger, Workflow,
};

use crate::{
    command::{AuthKind, CliCommand, ContractKind},
    context::CliContext,
    ipfs::IpfsFile,
    output::{
        OutputComponentUpload, OutputContractInstantiate, OutputContractUpload,
        OutputOperatorSetSigningKey, OutputServiceUpload,
    },
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Install rustls crypto provider before any TLS operations
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    tracing_init();

    let ctx = CliContext::new().await;

    match ctx.command.clone() {
        CliCommand::OperatorSetSigningKey {
            service_manager_address,
            evm_operator_address,
            stake_registry_address,
            wavs_instance,
            weight,
            wavs_url,
            args,
        } => {
            let service_manager_address =
                ctx.parse_address(&service_manager_address).await.unwrap();
            let stake_registry_address = ctx.parse_address(&stake_registry_address).await.unwrap();

            let service_manager = ServiceManager::Cosmos {
                chain: ctx.args().chain.clone(),
                address: service_manager_address.clone().try_into().unwrap(),
            };

            let body = serde_json::to_string(&GetSignerRequest { service_manager }).unwrap();

            let url = wavs_url.join("services/signer").unwrap();
            let SignerResponse::Secp256k1 {
                evm_address: evm_signing_key_address,
                hd_index: _,
            } = reqwest::Client::new()
                .post(url.clone())
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            let client = ctx.signing_client().await.unwrap();

            // Parse EVM addresses
            let evm_operator_address: EvmAddr = evm_operator_address.parse().expect(&format!(
                "Invalid operator EVM address '{}'",
                evm_operator_address
            ));

            let evm_signing_key_address: EvmAddr =
                evm_signing_key_address.parse().expect(&format!(
                    "Invalid signing key EVM address '{}'",
                    evm_signing_key_address
                ));

            // Parse weight as Uint256
            let weight_uint: Uint256 = weight
                .parse()
                .unwrap_or_else(|e| panic!("Invalid weight '{}': {}", weight, e));

            // Create the SetSigningKey message
            // TODO: move this to middleware docker cli
            let set_signing_key_msg = serde_json::json!({
                "set_signing_key": {
                    "operator": evm_operator_address.to_string(),
                    "signing_key": evm_signing_key_address.to_string(),
                    "weight": weight_uint.to_string()
                }
            });

            let tx_resp = client
                .contract_execute(
                    &service_manager_address.into(),
                    &set_signing_key_msg,
                    vec![],
                    None,
                )
                .await
                .unwrap();

            let service_manager_tx_hash = tx_resp.txhash;

            // Create the SetOperatorDetails message
            // TODO: move this to middleware docker cli
            let set_operator_details_msg = serde_json::json!({
                "set_operator_details": {
                    "operator": evm_operator_address.to_string(),
                    "signing_key": evm_signing_key_address.to_string(),
                    "weight": weight_uint.to_string()
                }
            });

            let tx_resp = client
                .contract_execute(
                    &stake_registry_address,
                    &set_operator_details_msg,
                    vec![],
                    None,
                )
                .await
                .unwrap();

            let stake_registry_tx_hash = tx_resp.txhash;

            let mut output = args.output();

            output.output_filename =
                if let Some((name, ext)) = output.output_filename.rsplit_once('.') {
                    format!("{name}-{wavs_instance}.{ext}")
                } else {
                    format!("{}-{wavs_instance}", output.output_filename)
                };

            output
                .write(OutputOperatorSetSigningKey {
                    service_manager_tx_hash,
                    stake_registry_tx_hash,
                    evm_operator_address,
                    evm_signing_key_address,
                })
                .await
                .unwrap();
        }
        CliCommand::TelegramGetWebhook { args: _ } => {
            let webhook_info = ctx.tg_messenger().get_webhook().await.unwrap();

            println!("{webhook_info:#?}");
        }
        CliCommand::TelegramSetWebhook {
            webhook,
            webhook_secret,
            args: _,
        } => {
            ctx.tg_messenger()
                .set_webhook(webhook.as_str(), &webhook_secret)
                .await
                .unwrap();

            tracing::info!("Webhook set successfully");
        }
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
            auth_address,
            auth_kind,
            args,
            code_id,
        } => {
            let client = ctx.signing_client().await.unwrap();

            let auth = match auth_kind {
                AuthKind::ServiceManager => {
                    tg_contract_api::payments::msg::Auth::ServiceManager(match auth_address {
                        Some(addr) => ctx.parse_address(&addr).await.unwrap().to_string(),
                        None => {
                            panic!("Service manager auth requires an address to be provided")
                        }
                    })
                }
                AuthKind::User => tg_contract_api::payments::msg::Auth::Admin(match auth_address {
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
            activate,
        } => {
            let output_directory = args.output().directory();

            let contract_payments_instantiation_file =
                output_directory.join(contract_payments_instantiation_file);
            let component_operator_cid_file = output_directory.join(component_operator_cid_file);
            let component_aggregator_cid_file =
                output_directory.join(component_aggregator_cid_file);
            let middleware_instantiation_file =
                output_directory.join(middleware_instantiation_file);

            fn strip_trailing_slash(url: &Url) -> String {
                let s = url.as_str();
                if s.ends_with('/') {
                    s[..s.len() - 1].to_string()
                } else {
                    s.to_string()
                }
            }

            let ipfs_api_url = strip_trailing_slash(&ipfs_api_url);
            let ipfs_gateway_url = strip_trailing_slash(&ipfs_gateway_url);
            let aggregator_url = strip_trailing_slash(&aggregator_url);

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
                env_keys: ["WAVS_ENV_OPERATOR_TELEGRAM_BOT_TOKEN".to_string()]
                    .into_iter()
                    .collect(),
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
                url: aggregator_url,
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
                status: if activate {
                    wavs_types::ServiceStatus::Active
                } else {
                    wavs_types::ServiceStatus::Paused
                },
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

        CliCommand::OperatorDeleteService {
            args,
            service_manager_address,
            wavs_url,
        } => {
            let req = wavs_types::DeleteServicesRequest {
                service_managers: vec![ServiceManager::Cosmos {
                    chain: args.chain,
                    address: service_manager_address.parse().unwrap(),
                }],
            };

            reqwest::Client::new()
                .delete(wavs_url.join("services").unwrap())
                .json(&req)
                .send()
                .await
                .unwrap()
                .error_for_status()
                .unwrap();
        }
    }
}
