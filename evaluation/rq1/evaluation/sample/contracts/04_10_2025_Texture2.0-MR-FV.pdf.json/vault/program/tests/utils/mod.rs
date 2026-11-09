use chrono::Utc;
use price_proxy::instruction::{CreatePriceFeed, WritePrice};
use price_proxy::state::price_feed::{PriceFeed, PriceFeedParams};
use serde::{Deserialize, Serialize};
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use solana_program_test::{
    find_file, read_file, BanksClient, BanksClientError, ProgramTest, ProgramTestBanksClientExt,
    ProgramTestContext,
};
use solana_sdk::account::Account;
use solana_sdk::signature::{read_keypair_file, Keypair, Signer};
use solana_sdk::transaction::Transaction;
use spl_token::state::Account as TokenAccount;
use std::str::FromStr;
use super_lendy::state::reserve::Reserve;
use texture_common::account::PodAccount;
use texture_common::math::Decimal;

pub mod setup;
pub mod superlendy_executor;
pub mod vault_executor;

pub const LAMPORTS_PER_USDC: u64 = 1_000_000;
pub const USDC_DECIMALS: u8 = 6;

pub fn admin_keypair() -> Keypair {
    read_keypair_file("../local-test/keys/LSDrcJsWUHMGJ6aQqHeapNftxhftkzR5SFs3YghQe4Y.json")
        .unwrap()
}

pub fn borrow_keypair() -> Keypair {
    read_keypair_file("../local-test/keys/uborhWCsojScXvx1dM5YaGLj4nPVXUYam57szVdVv8H.json")
        .unwrap()
}

pub fn texture_config_keypair() -> Keypair {
    read_keypair_file("../local-test/keys/gLoBanTpd5VuvyCpYjvYNudFREwLqFy418fGuuXUJfX.json")
        .unwrap()
}

pub fn price_feed_authority() -> Keypair {
    Keypair::from_bytes(&[
        52, 254, 138, 187, 39, 192, 47, 12, 4, 57, 178, 237, 196, 135, 132, 244, 11, 33, 144, 80,
        97, 206, 203, 254, 61, 196, 206, 68, 22, 82, 58, 177, 170, 146, 243, 184, 250, 17, 143, 33,
        84, 157, 252, 68, 214, 251, 10, 26, 48, 55, 182, 3, 189, 230, 147, 82, 74, 195, 46, 54, 62,
        16, 153, 234,
    ])
    .unwrap()
}

pub fn init_program_test() -> ProgramTest {
    tracing_init();

    tracing::info!("init program test...");

    let mut program_test = ProgramTest::default();

    let data = read_file(find_file("super_lendy.so").unwrap());
    program_test.add_account(
        super_lendy::SUPER_LENDY_ID,
        Account {
            lamports: 10 ^ 28,
            data,
            owner: solana_sdk::bpf_loader::id(),
            executable: true,
            rent_epoch: 0,
        },
    );

    let data = read_file(find_file("price_proxy.so").unwrap());
    program_test.add_account(
        price_proxy::ID,
        Account {
            lamports: 10 ^ 28,
            data,
            owner: solana_sdk::bpf_loader::id(),
            executable: true,
            rent_epoch: 0,
        },
    );

    let data = read_file(find_file("curvy.so").unwrap());
    program_test.add_account(
        Pubkey::from_str("CurvyNqr6HXwWUHk7MfDk7rqL3a4Kodkz8BkdBhep7ed").unwrap(),
        Account {
            lamports: 10 ^ 28,
            data,
            owner: solana_sdk::bpf_loader::id(),
            executable: true,
            rent_epoch: 0,
        },
    );

    let data = read_file(find_file("vault.so").unwrap());
    program_test.add_account(
        vault::VAULT_ID,
        Account {
            lamports: 10 ^ 28,
            data,
            owner: solana_sdk::bpf_loader::id(),
            executable: true,
            rent_epoch: 0,
        },
    );
    program_test
}

pub fn init_token_accounts(runner: &mut ProgramTest, mint: &Pubkey) {
    macro_rules! token_data_path {
        ($data:literal) => {
            format!(
                concat!(
                    std::env!("CARGO_MANIFEST_DIR"),
                    "/../local-test/tokens/{}-",
                    $data,
                    ".json"
                ),
                mint
            )
        };
    }

    let paths = [
        token_data_path!("mint"),
        token_data_path!("borrower-wallet"),
        token_data_path!("lender-wallet"),
    ];

    for path in paths.iter() {
        let account_info: JSONAccountInfo = {
            let file_data = std::fs::read(path).unwrap();
            serde_json::from_slice(&file_data).unwrap()
        };
        runner.add_account_with_base64_data(
            Pubkey::from_str(&account_info.pubkey).unwrap(),
            account_info.account.lamports,
            Pubkey::from_str(&account_info.account.owner).unwrap(),
            &account_info.account.data[0],
        )
    }
}

pub async fn add_price_feed_acc(runner: &mut ProgramTest, file: &str) -> Pubkey {
    let feed_data_path = format!(
        concat!(
            std::env!("CARGO_MANIFEST_DIR"),
            "/../local-test/price-feeds/{}",
            ".json"
        ),
        file
    );
    let account_info: JSONAccountInfo = {
        let file_data = std::fs::read(feed_data_path).unwrap();
        serde_json::from_slice(&file_data).unwrap()
    };

    let price_feed = Pubkey::from_str(&account_info.pubkey).unwrap();
    runner.add_account_with_base64_data(
        price_feed,
        account_info.account.lamports,
        Pubkey::from_str(&account_info.account.owner).unwrap(),
        &account_info.account.data[0],
    );
    price_feed
}

pub async fn add_curve_acc(runner: &mut ProgramTest, file: &str) -> Pubkey {
    let feed_data_path = format!(
        concat!(
            std::env!("CARGO_MANIFEST_DIR"),
            "/../local-test/irm/{}",
            ".json"
        ),
        file
    );
    let account_info: JSONAccountInfo = {
        let file_data = std::fs::read(feed_data_path).unwrap();
        serde_json::from_slice(&file_data).unwrap()
    };

    let curve = Pubkey::from_str(&account_info.pubkey).unwrap();
    runner.add_account_with_base64_data(
        curve,
        account_info.account.lamports,
        Pubkey::from_str(&account_info.account.owner).unwrap(),
        &account_info.account.data[0],
    );
    curve
}

pub async fn create_price_feed(
    context: &mut ProgramTestContext,
    params: PriceFeedParams,
) -> Pubkey {
    let authority = price_feed_authority();
    let price_feed_keypair = Keypair::new();
    let price_feed = price_feed_keypair.pubkey();

    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");

    let tx = Transaction::new_signed_with_payer(
        &[CreatePriceFeed {
            price_feed,
            authority: authority.pubkey(),
            source_address: authority.pubkey(),
            transform_source_address: authority.pubkey(),
            params,
        }
        .into_instruction()],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority, &price_feed_keypair],
        blockhash,
    );
    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("create_price_feed");
    price_feed
}

pub async fn write_price(
    context: &mut ProgramTestContext,
    price_feed: Pubkey,
    authority: &Keypair,
    price: Decimal,
    price_timestamp: i64,
) -> Result<(), BanksClientError> {
    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");

    let tx = Transaction::new_signed_with_payer(
        &[WritePrice {
            price_feed,
            authority: authority.pubkey(),
            price,
            price_timestamp,
        }
        .into_instruction()],
        Some(&context.payer.pubkey()),
        &[&context.payer, authority],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

pub async fn update_prices(context: &mut ProgramTestContext, reserves: &[Pubkey]) {
    let authority = price_feed_authority();
    let now = Utc::now().timestamp();
    for reserve in reserves {
        let reserve_acc = get_account(&mut context.banks_client, *reserve)
            .await
            .expect("get Reserve");
        let unpacked_reserve =
            Reserve::try_from_bytes(&reserve_acc.data).expect("unpacking Reserve");
        let price_feed_key = unpacked_reserve.config.market_price_feed;
        let price_feed_acc = get_account(&mut context.banks_client, price_feed_key)
            .await
            .expect("get PriceFeed");
        let price_feed =
            PriceFeed::try_from_bytes(&price_feed_acc.data).expect("unpacking PriceFeed");
        write_price(
            context,
            price_feed_key,
            &authority,
            price_feed.try_price().unwrap(),
            now - 3, // -3 is to avoid appearing in the future
        )
        .await
        .expect("update price feed")
    }
}

pub const LAMPORTS: u64 = 1_000_000_000_000_000;

pub async fn get_account(
    banks_client: &mut BanksClient,
    address: Pubkey,
) -> std::io::Result<Account> {
    banks_client
        .get_account(address)
        .await?
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "account not found"))
}

pub async fn get_token_account(
    banks_client: &mut BanksClient,
    address: Pubkey,
) -> std::io::Result<TokenAccount> {
    let acc = get_account(banks_client, address).await?;
    TokenAccount::unpack_from_slice(acc.data.as_ref())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
}

pub async fn create_associated_token_account(
    context: &mut ProgramTestContext,
    wallet: &Keypair,
    token_mint: &Pubkey,
) -> Result<Pubkey, BanksClientError> {
    let recent_blockhash = context.last_blockhash;

    let ix = spl_associated_token_account::instruction::create_associated_token_account(
        &context.payer.pubkey(),
        &wallet.pubkey(),
        token_mint,
        &spl_token::ID,
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        recent_blockhash,
    );

    // connection.send_and_confirm_transaction(&tx)?;
    context.banks_client.process_transaction(tx).await.unwrap();

    Ok(spl_associated_token_account::get_associated_token_address(
        &wallet.pubkey(),
        token_mint,
    ))
}

pub trait Runner {
    fn add_account(&mut self, key: Pubkey, account: Account) -> &mut Self;

    fn add_native_wallet(&mut self, key: Pubkey, lamports: u64) -> &mut Self {
        self.add_account(
            key,
            Account {
                lamports,
                data: Vec::new(),
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        )
    }

    fn add_account_with(&mut self, key: Pubkey, owner: Pubkey, f: impl FnOnce() -> Vec<u8>) {
        let data = f();
        let mut account = Account::new(LAMPORTS, data.len(), &owner);
        account.data = data;
        self.add_account(key, account);
    }
}

impl Runner for ProgramTest {
    fn add_account(&mut self, key: Pubkey, account: Account) -> &mut Self {
        self.add_account(key, account);
        self
    }
}

pub fn tracing_init() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
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
                    ",solana_program_test=debug",
                    ",solana_runtime::message_processor::stable_log=debug",
                    ",solana_program_runtime=warn",
                    ",solana_program=warn",
                    ",solana_runtime=warn",
                    ",solana_metrics=warn",
                    ",tarpc=error",
                )
                .to_owned(),
            };
            subscriber.with(Targets::from_str(&targets).unwrap())
        };

        subscriber.try_init().unwrap();
    });
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JSONAccountInfo {
    account: JSONAccount,
    pubkey: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JSONAccount {
    data: Vec<String>,
    executable: bool,
    lamports: u64,
    owner: String,
    rent_epoch: u16,
    space: Option<u16>,
}
