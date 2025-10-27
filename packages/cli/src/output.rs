use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::command::ContractKind;

pub struct Output {
    pub path: PathBuf,
    pub format: OutputFormat,
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
#[clap(rename_all = "snake_case")]
pub enum OutputFormat {
    Json,
}

impl Output {
    pub async fn write(&self, data: OutputData) -> Result<()> {
        match self.format {
            OutputFormat::Json => {
                let json_data = serde_json::to_string_pretty(&data)?;
                tokio::fs::write(&self.path, json_data).await?;
            }
        }
        tracing::info!("Output written to {}", self.path.display());

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged, rename_all = "snake_case")]
pub enum OutputData {
    ContractUpload {
        kind: ContractKind,
        code_id: u64,
        tx_hash: String,
    },
    ContractInstantiate {
        kind: ContractKind,
        address: String,
        tx_hash: String,
    },
}
