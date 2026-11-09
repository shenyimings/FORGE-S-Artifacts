use std::path::PathBuf;

use derive_more::FromStr;
use solana_sdk::{commitment_config::CommitmentLevel, pubkey::Pubkey};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Opts {
    /// URL of RPC Solana interface.
    #[structopt(
        long,
        short,
        default_value = "http://localhost:8899",
        env = "SOLANA_RPC"
    )]
    pub url: String,

    #[structopt(long, default_value = "confirmed")]
    pub commitment: CommitmentLevel,

    /// Keypair to use for signing instructions.
    #[structopt(long, short = "k", default_value)]
    pub authority: KeypairPath,

    /// Priority fee in microlamports. For priority_rate=1 you pay 0.2 (1) priority lamports for one ix, for 10_000 - 2_000.
    #[structopt(long)]
    pub priority_fee: Option<u64>,

    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Command {
    /// Creates Curve account. Requires ADMIN privileges.
    CreateCurve {
        /// Curve name
        #[structopt(long)]
        name: String,
        /// Human-readable formula
        #[structopt(long)]
        formula: String,
        #[structopt(long, default_value = "6")]
        decimals: u8,
        /// Source file (data in CSV)
        #[structopt(long, parse(from_os_str))]
        csv: PathBuf,
    },
    /// Alters Curve account
    AlterCurve {
        /// Curve account
        #[structopt(long)]
        curve: Pubkey,
        /// Curve name
        #[structopt(long)]
        name: Option<String>,
        /// Human-readable formula
        #[structopt(long)]
        formula: Option<String>,
        #[structopt(long)]
        decimals: Option<u8>,
        /// Source file (data in CSV)
        #[structopt(long, parse(from_os_str))]
        csv: Option<PathBuf>,
    },
    /// Deletes Curve account
    DeleteCurve {
        /// Curve account
        #[structopt(long)]
        curve: Pubkey,
    },
    /// Get Curve
    Curve {
        /// Curve account
        #[structopt(long)]
        curve: Pubkey,
    },
    /// Get all Curves
    Curves,
    /// Calculate and print Y value for given X on given curve
    CalcY {
        /// Curve account
        #[structopt(long)]
        curve: Pubkey,
        /// X coordinate
        #[structopt(long)]
        x: f64,
    },
}

#[derive(FromStr)]
pub struct KeypairPath(pub PathBuf);

impl Default for KeypairPath {
    fn default() -> Self {
        let mut path = dirs_next::home_dir().expect("home dir");
        path.extend([".config", "solana", "id.json"]);
        Self(path)
    }
}

impl ToString for KeypairPath {
    fn to_string(&self) -> String {
        self.0.to_str().expect("non unicode").to_string()
    }
}
