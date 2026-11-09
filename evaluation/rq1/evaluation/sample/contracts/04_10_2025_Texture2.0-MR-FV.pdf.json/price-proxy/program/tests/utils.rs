use std::sync::Arc;

use async_once_cell::OnceCell;
use price_proxy_client::PriceProxyClient;
use pyth_solana_receiver_sdk::PYTH_PUSH_ORACLE_ID;
use tracing::debug;

pub use dev_utils::*;

pub const PYTH_CONFIG: Pubkey = pubkey!("DaWUKXCyXsnzcvLUyeJRWou8KTn7XtadgTsdhJ6RHS7b");
pub const PYTH_GURDIAN_SET: Pubkey = pubkey!("5gxPdahvSzcKySxXxPuRXZZ9s6h8hZ88XDVKavWpaQGn");

pub const SB_SOL_PRICE_SOURCE: Pubkey = pubkey!("GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR");
pub const SB_RAY_PRICE_SOURCE: Pubkey = pubkey!("2oALNZVi5czyHvKbnjE4Jf2gR7dNp1FBpEGaq4PzVAf7");
pub const SOL_RESERVE_SOURCE: Pubkey = pubkey!("4c8DmkmxmjdN1UPXQ6z5LXKx3kRByCE76PY3PVWLGhfo");
pub const USDC_RESERVE_SOURCE: Pubkey = pubkey!("BHvvvAHZYBTuUR5HnfSDR9Go1VnhcA1eLvUs9yfTxKey");

pub async fn init_test() -> TestContext {
    tracing_init();

    struct Context {
        validator: TestValidator,
        payer: Arc<Keypair>,
        sys_admin: Arc<Keypair>,
    }

    static CTX: OnceCell<Context> = OnceCell::new();

    let ctx = CTX
        .get_or_init(async {
            assert_cmd::Command::new("cargo")
                .current_dir(workspace_dir())
                .args(["build-sbf"])
                .assert()
                .success();

            let (mut solana_genesis, sys_admin) = init_solana_genesis();
            solana_genesis.add_program_with_path(
                price_proxy::ID,
                format!("{}/price_proxy.so", default_sbf_deploy_dir()).into(),
            );
            solana_genesis.add_program_with_path(
                pyth_solana_receiver_sdk::id(),
                concat!(
                    std::env!("CARGO_MANIFEST_DIR"),
                    "/../local-test/programs/pyth_solana_receiver.so"
                )
                .into(),
            );
            solana_genesis.add_program_with_path(
                PYTH_PUSH_ORACLE_ID,
                concat!(
                    std::env!("CARGO_MANIFEST_DIR"),
                    "/../local-test/programs/pyth_push_oracle.so"
                )
                .into(),
            );
            solana_genesis.add_program_with_path(
                pubkey!("HDwcJBJXjL9FpJ7UBsYBtaDjsBUhuLCUYoz3zr8SWWaQ"),
                concat!(
                    std::env!("CARGO_MANIFEST_DIR"),
                    "/../local-test/programs/wormhole.so"
                )
                .into(),
            );

            add_price_feed_accounts(
                &mut solana_genesis,
                SB_SOL_PRICE_SOURCE,
                "switchboard-sol-price",
            );
            add_price_feed_accounts(
                &mut solana_genesis,
                SB_RAY_PRICE_SOURCE,
                "switchboard-ray-price",
            );
            add_price_feed_accounts(&mut solana_genesis, SOL_RESERVE_SOURCE, "sol-reserve");
            add_price_feed_accounts(&mut solana_genesis, USDC_RESERVE_SOURCE, "usdc-reserve");

            add_price_feed_accounts(&mut solana_genesis, PYTH_CONFIG, "pyth-config");
            add_price_feed_accounts(&mut solana_genesis, PYTH_GURDIAN_SET, "pyth-guardian-set");

            let (validator, payer) = solana_genesis.start_async().await;
            debug!(rpc = %validator.rpc_url(), "solana validator started");

            Context {
                validator,
                payer: Arc::new(payer),
                sys_admin: Arc::new(sys_admin),
            }
        })
        .await;

    TestContext {
        price_proxy: PriceProxyClient {
            rpc: ctx.validator.get_async_rpc_client(),
            authority: Keypair::from_bytes(&ctx.sys_admin.to_bytes()).unwrap(),
            priority_fee: None,
            show_spinner: true,
        },
        payer: ctx.payer.clone(),
    }
}

pub struct TestContext {
    pub price_proxy: PriceProxyClient,
    #[allow(dead_code)]
    pub payer: Arc<Keypair>,
}

#[allow(dead_code)]
pub fn add_price_feed_accounts(genesis: &mut TestValidatorGenesis, pubkey: Pubkey, file: &str) {
    let feed_data_path = format!(
        concat!(
            std::env!("CARGO_MANIFEST_DIR"),
            "/../local-test/source-accounts/{}",
            ".json"
        ),
        file
    );

    genesis
        .add_accounts_from_json_files(&[AccountInfo {
            address: Some(pubkey),
            filename: feed_data_path.as_str(),
        }])
        .expect("add_accounts_from_json_files");
}
