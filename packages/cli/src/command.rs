use crate::output::{Output, OutputFormat};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use tg_utils::path::repo_root;
use wavs_types::ChainKey;

#[derive(Clone, Parser)]
#[command(version, about, long_about = None)]
pub enum CliCommand {
    /// Generate mnemonics for the .env file
    UploadContract {
        #[arg(long)]
        kind: ContractKind,

        #[clap(flatten)]
        args: CliArgs,
    },
    InstantiatePayments {
        #[arg(long)]
        code_id: u64,

        #[arg(long, default_values = ["untrn", "uatom"], num_args = 1..)]
        allowed_denoms: Vec<String>,

        /// If AuthKind is User, then this is the user address
        /// and None means the CLI mnemonic address
        #[arg(long)]
        auth: Option<String>,

        #[arg(long, default_value_t = AuthKind::User)]
        auth_kind: AuthKind,

        #[clap(flatten)]
        args: CliArgs,
    },
    FaucetTap {
        /// if not supplied, will be the one in CLI_MNEMONIC
        addr: Option<String>,
        /// if not supplied, will be the default
        amount: Option<u128>,
        /// if not supplied, will be the default
        denom: Option<String>,
        #[clap(flatten)]
        args: CliArgs,
    },
}

// common args for several commands
#[derive(Clone, Debug, Parser)]
pub struct CliArgs {
    #[clap(long, default_value = "cosmos:pion-1")]
    pub chain: ChainKey,

    /// Filename for outputting any generated files
    /// which will be written in to `builds/cli/`
    #[clap(long, default_value = "output.json")]
    pub output_filename: String,

    /// Output format for any generated files
    #[clap(long, value_enum, default_value_t = OutputFormat::Json)]
    pub output_format: OutputFormat,
}

impl CliArgs {
    pub fn output(&self) -> Output {
        let output_path = repo_root()
            .expect("could not determine repo root")
            .join("builds")
            .join("deployments")
            .join(&self.output_filename);

        // Ensure the output directory exists
        std::fs::create_dir_all(output_path.parent().unwrap()).unwrap_or_else(|_| {
            panic!(
                "Failed to create output directory: {}",
                output_path.parent().unwrap().display()
            )
        });

        Output {
            path: output_path,
            format: self.output_format,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ValueEnum)]
#[clap(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ContractKind {
    Payments,
}

impl std::fmt::Display for ContractKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ContractKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Payments => "payments",
        }
    }
    pub async fn wasm_bytes(&self) -> Vec<u8> {
        let path = repo_root()
            .unwrap()
            .join("builds")
            .join("contracts")
            .join(&format!("tg_contract_{}.wasm", self.as_str()));

        tokio::fs::read(&path)
            .await
            .unwrap_or_else(|_| panic!("Failed to read wasm bytes at: {}", path.to_string_lossy()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ValueEnum)]
#[clap(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuthKind {
    User,
    ServiceManager,
}

impl std::fmt::Display for AuthKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AuthKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::User => "user",
            Self::ServiceManager => "service-manager",
        }
    }
}
