use std::path::{Path, PathBuf};
use tg_utils::path::repo_root;
use tracing::{debug, info, instrument};

use crate::client::TestPool;

static PAYMENTS_CODE_ID: tokio::sync::OnceCell<u64> = tokio::sync::OnceCell::const_new();

pub struct CodeId {}

impl CodeId {
    #[instrument]
    pub async fn new_payments() -> u64 {
        *PAYMENTS_CODE_ID.get_or_init(upload_payments).await
    }
}

async fn upload_payments() -> u64 {
    upload(wasm_path("payments")).await
}

#[instrument(skip(wasm_path), fields(path = %wasm_path.as_ref().display()))]
async fn upload(wasm_path: impl AsRef<Path>) -> u64 {
    let wasm_path = wasm_path.as_ref();

    info!("Reading WASM file");
    let wasm_bytes = tokio::fs::read(&wasm_path)
        .await
        .unwrap_or_else(|_| panic!("Failed to read {}", wasm_path.display()));

    debug!(size_bytes = wasm_bytes.len(), "WASM file loaded");

    let pool = TestPool::get().await;
    let client = pool.pool.get().await.unwrap();

    debug!("Uploading contract to chain");
    let code_id = client
        .contract_upload_file(wasm_bytes, None)
        .await
        .unwrap()
        .0;

    info!(code_id, "Contract uploaded successfully");

    code_id
}

fn wasm_path(contract: &str) -> PathBuf {
    repo_root()
        .unwrap()
        .join("builds")
        .join("contracts")
        .join(format!("tg_contract_{contract}.wasm"))
}
