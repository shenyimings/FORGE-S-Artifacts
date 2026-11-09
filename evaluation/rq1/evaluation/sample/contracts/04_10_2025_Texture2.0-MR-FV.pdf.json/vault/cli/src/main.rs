use std::fmt::Display;
use std::path::PathBuf;

use anyhow::anyhow;
use derive_more::FromStr;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::read_keypair_file;
use structopt::StructOpt;

use vault::state::vault::DistributionItem;
use vlt::app::App;

#[derive(FromStr)]
struct KeypairPath(PathBuf);

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct VaultArgs {
    /// URL of RPC Solana interface.
    #[structopt(
        long,
        short,
        default_value = "http://localhost:8899",
        env = "SOLANA_RPC"
    )]
    pub url: String,

    /// Keypair to use for signing instructions and to define Lender / Borrower role.
    #[structopt(long, short = "k", default_value)]
    pub authority: KeypairPath,

    /// Priority fee in microlamports. For priority_rate=1 you pay 0.2 (1) priority lamports for one ix, for 10_000 - 2_000.
    #[structopt(long)]
    priority_fee: Option<u64>,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Command {
    /// Get contract version
    ContractVersion,
    /// Creates Vault
    CreateVault {
        /// Vault name
        #[structopt(long)]
        name: String,
        /// Vault description
        #[structopt(long)]
        description: String,
        /// Vault liquidity mint
        #[structopt(long)]
        mint: Pubkey,
        /// Authority who can change percentage in distribution (usually bot).
        #[structopt(long)]
        distribution_manager: Pubkey,
        /// Vault distribution
        #[structopt(long)]
        distribution: Vec<DistributionItem>,
        /// Vault curator
        #[structopt(long)]
        curator: Pubkey,
        /// Periodic fee BPS
        #[structopt(long)]
        periodic_fee_rate_bps: u16,
    },
    /// Change Vault name, description or distribution
    AlterVault {
        /// Vault to change
        #[structopt(long)]
        vault: Pubkey,
        /// Vault name
        #[structopt(long)]
        name: Option<String>,
        /// Vault description
        #[structopt(long)]
        description: Option<String>,
        /// Vault distribution
        #[structopt(long)]
        distribution: Option<Vec<DistributionItem>>,
        /// Authority who can change percentage in distribution (usually bot).
        #[structopt(long)]
        distribution_manager: Option<Pubkey>,
        /// Periodic fee BPS
        #[structopt(long)]
        periodic_fee_rate_bps: Option<u16>,
    },
    /// Propose new item in to distribution. It must brew 3 days before it can be added in to Vault.
    ProposeDistributionItem {
        /// Vault to add item to
        #[structopt(long)]
        vault: Pubkey,
        /// Reserve for new item to add
        #[structopt(long)]
        reserve: Pubkey,
        /// Index to store proposal
        #[structopt(long)]
        index: u8,
    },
    /// Adds new item (based on proposal) in to distribution with 0 share. Later this can be changed via AlterVault
    AddDistributionItem {
        /// Vault to add item to
        #[structopt(long)]
        vault: Pubkey,
        /// Index to apply proposal from
        #[structopt(long)]
        index: u8,
    },
    RemoveDistributionItem {
        /// Vault to remove item from
        #[structopt(long)]
        vault: Pubkey,
        /// Reserve to identify item
        #[structopt(long)]
        reserve: Pubkey,
    },
    /// Rebalance Vault
    Rebalance {
        /// Vault
        #[structopt(long)]
        vault: Pubkey,
    },
    /// List vaults
    Vaults,
    /// Create position
    CreatePosition {
        /// Vault
        #[structopt(long)]
        vault: Pubkey,
    },
    /// Deposit liqudity
    Deposit {
        /// Position
        #[structopt(long)]
        position: Pubkey,
        /// Amount
        #[structopt(long)]
        amount: u64,
    },
    /// Withdraw liqudity
    Withdraw {
        /// Position
        #[structopt(long)]
        position: Pubkey,
        /// VLP amount to return
        #[structopt(long)]
        vlp_amount: u64,
    },
    /// Close position
    ClosePosition {
        /// Position
        #[structopt(long)]
        position: Pubkey,
    },
    /// List positions
    ListPositions,
    /// Claim Vault fees
    ClaimFees {
        /// Vault
        #[structopt(long)]
        vault: Pubkey,
        /// Minimal acceptable liquidity amount to claim as fee
        #[structopt(long)]
        min_fee_amount: Option<u64>,
    },
    /// Init SPL token account to hold reward tokens for given pool. Call it from pools mgmt authority.
    InitRewardSupply {
        /// Vault to init reward supply for
        #[structopt(long)]
        vault: Pubkey,
        #[structopt(long)]
        reward_mint: Pubkey,
    },
    /// Set reward rule in a particular Vault
    SetRewardRule {
        /// Vault to set reward rule in
        #[structopt(long)]
        vault: Pubkey,
        /// Rule index (starting from 0) to set
        #[structopt(long)]
        index: usize,
        /// Rule name
        #[structopt(long)]
        name: String,
        /// Reward token mint
        #[structopt(long)]
        reward_mint: Pubkey,
        /// This is amount of reward token to give per each owned VLP per slot
        /// E.g. value 0.001 means that per each VLP User will earn 1 reward token each
        /// 1000 slots. So if he deposit 1 token and will hold this deposit 2000 slots he will earn 2
        /// reward tokens.
        #[structopt(long)]
        rate: f64,
    },
    /// Claim reward.
    ClaimReward {
        /// Position to claim rewards from
        #[structopt(long)]
        position: Pubkey,
        /// Mint of the reward token to claim
        #[structopt(long)]
        reward_mint: Pubkey,
    },
    /// Prints address of reward supply SPL wallet
    RewardSupplyAddr {
        /// Vault
        #[structopt(long)]
        vault: Pubkey,
        /// Mint of the reward token
        #[structopt(long)]
        reward_mint: Pubkey,
    },
    /// Used to deposit more reward tokens on contract managed rewards supply
    DepositReward {
        /// Vault
        #[structopt(long)]
        vault: Pubkey,
        /// Mint of the reward token
        #[structopt(long)]
        reward_mint: Pubkey,
        #[structopt(long)]
        amount: u64,
    },
    /// Used to withdraw reward tokens from reward supply. Call it with Vault curator authority
    WithdrawReward {
        /// Vault
        #[structopt(long)]
        vault: Pubkey,
        /// Mint of the reward token
        #[structopt(long)]
        reward_mint: Pubkey,
        #[structopt(long)]
        amount: u64,
    },
    /// Prints all existing SPL token account with rewards for given Vault
    RewardsBalances {
        /// Vault
        #[structopt(long)]
        vault: Pubkey,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let opt: VaultArgs = VaultArgs::from_args();

    let keypair = read_keypair_file(opt.authority.0)
        .map_err(|err| anyhow!("reading authority keypair: {}", err))
        .unwrap();
    let client = RpcClient::new_with_commitment(opt.url.clone(), CommitmentConfig::confirmed());

    let app = App {
        rpc: client,
        url: opt.url,
        authority: keypair,
        priority_fee: opt.priority_fee,
    };

    match opt.cmd {
        Command::ContractVersion {} => {
            app.contract_version().await;
        }
        Command::CreateVault {
            name,
            description,
            mint,
            distribution_manager,
            distribution,
            curator,
            periodic_fee_rate_bps,
        } => {
            if distribution.is_empty() {
                Err(anyhow!("Empty distribution is not allowed!"))?
            }

            app.create_vault(
                name,
                description,
                mint,
                distribution_manager,
                curator,
                distribution,
                periodic_fee_rate_bps,
            )
            .await;
        }
        Command::AlterVault {
            vault,
            name,
            description,
            distribution,
            distribution_manager,
            periodic_fee_rate_bps,
        } => {
            app.alter_vault(
                vault,
                name,
                description,
                distribution,
                distribution_manager,
                periodic_fee_rate_bps,
            )
            .await;
        }
        Command::ProposeDistributionItem {
            vault,
            reserve,
            index,
        } => {
            app.propose_distribution_item(vault, reserve, index).await;
        }
        Command::AddDistributionItem { vault, index } => {
            app.add_distribution_item(vault, index).await;
        }
        Command::RemoveDistributionItem { vault, reserve } => {
            app.remove_distribution_item(vault, reserve).await;
        }
        Command::Rebalance { vault } => {
            app.rebalance(vault).await;
        }
        Command::Vaults {} => {
            app.list_vaults().await;
        }
        Command::CreatePosition { vault } => {
            app.create_position(vault).await;
        }
        Command::Deposit { position, amount } => {
            app.deposit(position, amount).await;
        }
        Command::Withdraw {
            position,
            vlp_amount,
        } => {
            app.withdraw(position, vlp_amount).await;
        }
        Command::ClosePosition { position } => {
            app.close_position(position).await;
        }
        Command::ListPositions {} => {
            app.list_positions().await;
        }
        Command::ClaimFees {
            vault,
            min_fee_amount,
        } => {
            app.claim_fees(vault, min_fee_amount).await;
        }
        Command::SetRewardRule {
            vault,
            index,
            name,
            reward_mint,
            rate,
        } => {
            app.set_reward_rule(vault, index, name, reward_mint, rate)
                .await
        }
        Command::InitRewardSupply { vault, reward_mint } => {
            app.init_reward_supply(vault, reward_mint).await
        }
        Command::ClaimReward {
            position,
            reward_mint,
        } => app.claim_reward(position, reward_mint).await,
        Command::RewardSupplyAddr { vault, reward_mint } => {
            app.reward_supply_addr(vault, reward_mint)
        }
        Command::DepositReward {
            vault,
            reward_mint,
            amount,
        } => app.deposit_reward(vault, reward_mint, amount).await,
        Command::WithdrawReward {
            vault,
            reward_mint,
            amount,
        } => app.withdraw_reward(vault, reward_mint, amount).await,
        Command::RewardsBalances { vault } => app.list_rewards_balances(vault).await,
    }

    Ok(())
}

impl Default for KeypairPath {
    fn default() -> Self {
        let mut path = dirs_next::home_dir().expect("home dir");
        path.extend([".config", "solana", "id.json"]);
        Self(path)
    }
}

impl Display for KeypairPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self.0.to_str().expect("non unicode").to_string();
        f.write_str(&string)
    }
}
