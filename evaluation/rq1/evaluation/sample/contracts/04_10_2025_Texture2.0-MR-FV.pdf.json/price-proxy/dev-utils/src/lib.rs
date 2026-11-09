use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

pub use anyhow;
pub use assert_cmd;
pub use assert_matches;
pub use async_trait::async_trait;
pub use lazy_static::lazy_static;
pub use predicates;
pub use pretty_assertions;
pub use solana_client::nonblocking::rpc_client::RpcClient;
pub use solana_program::pubkey;
pub use solana_rpc::rpc::JsonRpcConfig;
pub use solana_sdk::{
    account::Account, account::AccountSharedData, program_error::ProgramError, program_pack::Pack,
    pubkey::Pubkey, signature::Keypair, signer::Signer,
};
pub use solana_test_validator::AccountInfo;
use solana_test_validator::UpgradeableProgramInfo;
pub use solana_test_validator::{TestValidator, TestValidatorGenesis};
pub use tokio;
pub use tracing;

pub use texture_common::account::{PodAccount, PodAccountError};
pub use texture_common::math::{
    decimal::{dec, dec_impl},
    Decimal,
};
pub const MSOL_MINT: Pubkey = pubkey!("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So");
pub const MSOL_DECIMALS: u8 = 9;

pub const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const USDC_DECIMALS: u8 = 6;

pub fn admin_keypair() -> &'static Keypair {
    lazy_static! {
        static ref KEYPAIR: Keypair = Keypair::from_bytes(&[
            98, 110, 166, 223, 232, 38, 154, 235, 128, 235, 118, 246, 179, 65, 162, 188, 76, 81,
            80, 233, 30, 187, 157, 19, 36, 18, 80, 117, 204, 203, 175, 61, 136, 254, 30, 151, 76,
            159, 44, 75, 16, 238, 41, 241, 136, 158, 116, 25, 73, 25, 53, 49, 221, 227, 155, 234,
            210, 153, 116, 215, 179, 9, 135, 110,
        ])
        .unwrap();
    }
    &KEYPAIR
}
pub fn admin() -> &'static Pubkey {
    lazy_static! {
        static ref KEY: Pubkey = admin_keypair().pubkey();
    }
    &KEY
}

pub fn init_solana_genesis() -> (TestValidatorGenesis, Keypair) {
    let mut genesis = TestValidatorGenesis::default();

    let mut rpc_cfg = JsonRpcConfig::default_for_test();
    rpc_cfg.enable_rpc_transaction_history = true;
    genesis.rpc_config(rpc_cfg);

    let sys_admin = Keypair::new();
    genesis.add_wallet(sys_admin.pubkey(), 1_000_000 * 1_000_000_000);

    genesis.add_wallet(*admin(), 1_000_000 * 1_000_000_000);

    genesis
        .add_token_mint(
            MSOL_MINT,
            sys_admin.pubkey(),
            MSOL_DECIMALS,
            1_000_000 * 1_000_000_000,
        )
        .unwrap();

    genesis
        .add_token_mint(
            USDC_MINT,
            sys_admin.pubkey(),
            USDC_DECIMALS,
            1_000_000 * 1_000_000,
        )
        .unwrap();

    (genesis, sys_admin)
}

pub fn workspace_dir() -> String {
    let output = std::process::Command::new("cargo")
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = std::path::Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_string_lossy().to_string()
}

pub fn workspace_target_dir() -> String {
    std::env::var("CARGO_TARGET_DIR").unwrap_or(format!("{}/target", workspace_dir()))
}

pub fn default_sbf_deploy_dir() -> String {
    format!("{}/deploy", workspace_target_dir())
}

pub fn tracing_init() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        if std::env::var("REMOVE_RUST_BACKTRACE")
            .map(|value| value == "1")
            .unwrap_or(false)
        {
            std::env::remove_var("RUST_BACKTRACE");
        }

        unsafe { backtrace_on_stack_overflow::enable() };

        use tracing_subscriber::filter::LevelFilter;
        use tracing_subscriber::fmt::Subscriber;
        use tracing_subscriber::util::SubscriberInitExt;

        let builder = Subscriber::builder();
        let builder = builder.with_max_level(LevelFilter::TRACE);

        let subscriber = builder.finish();
        let subscriber = {
            use std::env;
            use tracing_subscriber::{filter::Targets, layer::SubscriberExt};
            let targets = match env::var("RUST_LOG") {
                Ok(var) => var,
                Err(_) => concat!(
                    "debug",
                    ",solana_core=off",
                    ",solana_runtime=off",
                    ",solana_rpc=off",
                    ",solana_send_transaction_service=off",
                    ",solana_net_utils=off",
                    ",solana_accounts_db=off",
                    ",solana_ledger=off",
                    ",solana_metrics=off",
                    ",solana_poh=off",
                    ",solana_streamer=off",
                    ",solana_quic_client=off",
                    ",solana_gossip=off",
                    ",solana_perf=off",
                    ",solana_vote_program=off",
                    ",solana_program::fee_calculator=off",
                    ",solana_program_runtime::loaded_program=off",
                    ",solana_connection_cache=off",
                    ",solana_cost_model=off",
                    ",tokio_postgres=warn",
                    ",rpc=warn",
                    ",reqwest=warn",
                    ",hyper=warn",
                    ",rustls=warn",
                )
                .to_owned(),
            };
            subscriber.with(Targets::from_str(&targets).unwrap())
        };

        subscriber.try_init().unwrap();
    });
}

pub trait SolanaRunnerExt {
    fn minimum_balance(&self, data_len: usize) -> u64;
    fn add_account(&mut self, key: Pubkey, owner: Pubkey, data: Vec<u8>, lamports: u64);
    fn add_program_with_path(&mut self, id: Pubkey, path: PathBuf);

    fn add_account_min(&mut self, key: Pubkey, owner: Pubkey, data: Vec<u8>) {
        let lamports = self.minimum_balance(data.len());
        self.add_account(key, owner, data, lamports)
    }

    fn add_wallet(&mut self, key: Pubkey, lamports: u64) {
        self.add_account(key, solana_sdk::system_program::ID, vec![], lamports)
    }

    fn add_pack_account<S: Pack>(
        &mut self,
        key: Pubkey,
        owner: Pubkey,
        state: S,
    ) -> Result<(), ProgramError> {
        let mut data = vec![0; S::LEN];
        S::pack(state, &mut data)?;
        self.add_account_min(key, owner, data);
        Ok(())
    }

    fn add_token_mint(
        &mut self,
        mint: Pubkey,
        authority: Pubkey,
        decimals: u8,
        supply: u64,
    ) -> Result<(), ProgramError> {
        let state = spl_token::state::Mint {
            mint_authority: Some(authority).into(),
            supply,
            decimals,
            is_initialized: true,
            freeze_authority: Some(authority).into(),
        };
        self.add_pack_account(mint, spl_token::ID, state)
    }

    fn add_token_account(
        &mut self,
        key: Pubkey,
        mint: Pubkey,
        owner: Pubkey,
        amount: u64,
    ) -> Result<(), ProgramError> {
        let state = spl_token::state::Account {
            mint,
            owner,
            amount,
            delegate: None.into(),
            state: spl_token::state::AccountState::Initialized,
            is_native: None.into(),
            delegated_amount: 0,
            close_authority: None.into(),
        };
        self.add_pack_account(key, spl_token::ID, state)
    }

    fn add_pod_account<S: PodAccount>(
        &mut self,
        key: Pubkey,
        owner: Pubkey,
        params: S::InitParams,
    ) -> Result<(), S::InitError> {
        let mut data = vec![0; S::SIZE];
        S::init_bytes(&mut data, params)?;
        self.add_account_min(key, owner, data);
        Ok(())
    }
}

impl SolanaRunnerExt for TestValidatorGenesis {
    fn minimum_balance(&self, data_len: usize) -> u64 {
        self.rent.minimum_balance(data_len)
    }

    fn add_account(&mut self, key: Pubkey, owner: Pubkey, data: Vec<u8>, lamports: u64) {
        self.add_account(
            key,
            Account {
                lamports,
                data,
                owner,
                executable: false,
                rent_epoch: 0,
            }
            .into(),
        );
    }

    fn add_program_with_path(&mut self, id: Pubkey, path: PathBuf) {
        self.add_upgradeable_programs_with_path(&[UpgradeableProgramInfo {
            program_id: id,
            loader: solana_sdk::bpf_loader_upgradeable::id(),
            upgrade_authority: Pubkey::default(),
            program_path: path,
        }]);
    }
}
pub trait DbQueryResIsEmpty {
    fn is_empty(&self) -> bool;
}

impl<T> DbQueryResIsEmpty for Vec<T> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[async_trait]
pub trait SolanaRpcExt {
    async fn get_account(&self, addr: &Pubkey) -> anyhow::Result<Account>;

    // async fn get_token_mint(&self, addr: &Pubkey) -> anyhow::Result<spl_token::state::Mint> {
    //     let account = self.get_account(addr).await?;
    //     Ok(spl_token::state::Mint::unpack(&account.data)?)
    // }

    async fn get_pod_account<T: PodAccount>(&self, addr: &Pubkey) -> anyhow::Result<T> {
        let account = self.get_account(addr).await?;
        Ok(*T::try_from_bytes(&account.data)?)
    }
}

#[async_trait]
impl SolanaRpcExt for RpcClient {
    async fn get_account(&self, addr: &Pubkey) -> anyhow::Result<Account> {
        Ok(self.get_account(addr).await?)
    }
}

pub trait AssertCmdAssertExt {
    fn log_output(self, msg: impl Display) -> assert_cmd::assert::Assert;
    #[track_caller]
    fn json_output<'de, T: serde::Deserialize<'de>>(&'de self) -> T;
}
impl AssertCmdAssertExt for assert_cmd::assert::Assert {
    fn log_output(self, msg: impl Display) -> assert_cmd::assert::Assert {
        let output = self.get_output();
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::debug!(status = %output.status, %stdout, "{msg}");
        self
    }

    #[track_caller]
    fn json_output<'de, T: serde::Deserialize<'de>>(&'de self) -> T {
        serde_json::from_slice(&self.get_output().stdout).expect("parse cmd output as json")
    }
}
