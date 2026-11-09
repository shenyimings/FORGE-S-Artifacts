use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};
use async_once_cell::OnceCell;
use solana_sdk::signer::EncodableKey;
use tracing::debug;

pub use dev_utils::*;

pub async fn init_test() -> &'static TestContext {
    tracing_init();

    static CTX: OnceCell<(TestContext, TestValidator)> = OnceCell::new();

    let ctx = CTX
        .get_or_init(async {
            assert_cmd::Command::new("cargo")
                .current_dir(workspace_dir())
                .args(["build-sbf"])
                .assert()
                .success();

            let (mut solana_genesis, _sys_admin) = init_solana_genesis();
            solana_genesis.add_program_with_path(
                price_proxy::ID,
                format!("{}/price_proxy.so", default_sbf_deploy_dir()).into(),
            );

            let (validator, payer) = solana_genesis.start_async().await;
            debug!(rpc = %validator.rpc_url(), "solana validator started");

            let payer_keypair_file =
                format!("/tmp/texture/price-proxy-test/cli/{}.json", payer.pubkey());
            payer
                .write_to_file(&payer_keypair_file)
                .expect("write payer keypair file");

            (
                TestContext {
                    rpc_url: validator.rpc_url(),
                    payer_keypair_file,
                    payer,
                },
                validator,
            )
        })
        .await;

    &ctx.0
}

pub struct TestContext {
    pub rpc_url: String,
    pub payer: Keypair,
    pub payer_keypair_file: String,
}

impl TestContext {
    pub fn new_price_proxy_cmd<I, S>(&self, args: I) -> std::process::Command
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        let mut cli =
            std::process::Command::cargo_bin("price-proxy").expect("cargo run price-proxy");
        cli.args(["-u", &self.rpc_url, "-k", &self.payer_keypair_file])
            .stderr(std::process::Stdio::inherit())
            .args(args);
        cli
    }

    pub fn price_proxy_cmd_assert(&self, args: &[&str]) -> assert_cmd::assert::Assert {
        let mut cmd = self.new_price_proxy_cmd(args);
        debug!(?cmd, "start");
        cmd.assert().log_output("price-proxy")
    }

    #[track_caller]
    pub fn price_proxy_cmd_spawn(&self, args: &[&str]) -> tokio::process::Child {
        let cmd = self.new_price_proxy_cmd(args);
        debug!(?cmd, "spawn");
        tokio::process::Command::from(cmd)
            .kill_on_drop(true)
            .spawn()
            .unwrap()
    }
}
