use anyhow::Result;
use clap::ValueEnum;
use layer_climb::prelude::EvmAddr;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tg_utils::path::repo_root;
use wavs_types::{ComponentDigest, ServiceDigest};

use crate::command::{CliArgs, ComponentKind, ContractKind};

pub struct Output {
    pub output_filename: String,
    pub format: OutputFormat,
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
#[clap(rename_all = "snake_case")]
pub enum OutputFormat {
    Json,
}

impl CliArgs {
    pub fn output(&self) -> Output {
        Output {
            output_filename: self.output_filename.clone(),
            format: self.output_format,
        }
    }
}

impl Output {
    pub async fn write(&self, data: impl Serialize) -> Result<()> {
        let directory = self.directory();
        let file = directory.join(&self.output_filename);

        // Ensure the output directory exists
        std::fs::create_dir_all(&directory).unwrap_or_else(|_| {
            panic!("Failed to create output directory: {}", directory.display())
        });

        match self.format {
            OutputFormat::Json => {
                let json_data = serde_json::to_string_pretty(&data)?;
                tokio::fs::write(&file, json_data).await?;
            }
        }
        tracing::info!("Output written to {}", file.display());

        Ok(())
    }

    pub fn directory(&self) -> PathBuf {
        repo_root()
            .expect("could not determine repo root")
            .join("builds")
            .join("deployments")
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct OutputOperatorSetSigningKey {
    pub service_manager_tx_hash: String,
    pub stake_registry_tx_hash: String,
    pub evm_operator_address: EvmAddr,
    pub evm_signing_key_address: EvmAddr,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct OutputContractUpload {
    pub kind: ContractKind,
    pub code_id: u64,
    pub tx_hash: String,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct OutputContractInstantiate {
    pub kind: ContractKind,
    pub address: String,
    pub tx_hash: String,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct OutputComponentUpload {
    pub kind: ComponentKind,
    pub component: String,

    /// The hash of the file,
    pub digest: ComponentDigest,

    /// The content identifier (CID) of the uploaded file
    pub cid: String,

    /// The IPFS URI (e.g., "ipfs://Qm...")
    pub uri: String,

    /// The gateway URL for accessing the file via HTTP
    pub gateway_url: String,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct OutputServiceUpload {
    pub service: wavs_types::Service,

    /// The hash of the file,
    pub digest: ServiceDigest,

    /// The content identifier (CID) of the uploaded file
    pub cid: String,

    /// The IPFS URI (e.g., "ipfs://Qm...")
    pub uri: String,

    /// The gateway URL for accessing the file via HTTP
    pub gateway_url: String,
}
