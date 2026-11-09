use anyhow::anyhow;
use pyth_solana_receiver_sdk::price_update::get_feed_id_from_hex;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::borsh1::try_from_slice_unchecked;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::read_keypair_file;
use std::time::Duration;
use structopt::StructOpt;
use texture_common::math::{CheckedDiv, Decimal};
use tracing::{info, warn};

use price_proxy::state::price_feed::{FeedType, PriceFeedParams, SYMBOL_MAX_SIZE};
use price_proxy::state::stake_pool::StakePool;
use price_proxy_client::{
    get_account_with_retries, PriceFeedSignatureView, PriceProxyClient as App, PriceProxyClient,
};

mod opts;

#[tokio::main]
async fn main() {
    let _tracing_appender_guard = tracing_init();

    let opts = opts::Opts::from_args();

    let keypair = read_keypair_file(opts.authority.0)
        .map_err(|err| anyhow!("reading authority keypair: {}", err))
        .unwrap();
    let rpc = RpcClient::new_with_commitment(
        opts.url.clone(),
        CommitmentConfig {
            commitment: opts.commitment,
        },
    );

    let app = App {
        rpc,
        authority: keypair,
        priority_fee: opts.priority_fee,
        show_spinner: true,
    };

    match opts.cmd {
        opts::Command::CreatePriceFeed {
            feed_type,
            symbol,
            quote_symbol,
            logo_url,
            source,
            transform_source,
            source_address,
            transform_source_address,
            verification_level,
        } => {
            if symbol.len() > SYMBOL_MAX_SIZE {
                println!("Symbol name is too long. {} max.", SYMBOL_MAX_SIZE);
                return;
            }

            if feed_type == FeedType::Transform
                && (transform_source.is_none() || transform_source_address.is_none())
            {
                println!("transform source or source address is none");
                return;
            }

            let transform_source = if let Some(transform_source) = transform_source {
                transform_source
            } else {
                source
            };
            let transform_source_address =
                if let Some(transform_source_address) = transform_source_address {
                    transform_source_address
                } else {
                    source_address
                };

            let params = PriceFeedParams::new(
                feed_type,
                &symbol,
                quote_symbol,
                verification_level,
                &logo_url,
                source,
                transform_source,
            );
            let created_price_feed = app
                .create_price_feed(params, source_address, transform_source_address)
                .await
                .expect("create price-feed");
            println_cmd_out!(&created_price_feed);
        }
        opts::Command::PriceFeed { key } => {
            let price_feed = app.price_feed(&key).await.expect("get price-feed");
            println_cmd_out!(&price_feed);
        }
        opts::Command::PriceFeeds { key: keys } => {
            let res = if keys.is_empty() {
                app.price_feeds().await
            } else {
                app.price_feeds_by_key(&keys).await
            };
            let price_feeds = res.expect("get price-feeds");
            println_cmd_out!(&price_feeds);
        }
        opts::Command::WritePrice { key, price } => {
            let signature = app
                .write_price(key, price, chrono::Utc::now().timestamp())
                .await
                .expect("write price");
            println_cmd_out!(&signature);
        }
        opts::Command::ForcePriceTimestamp { period, key: keys } => {
            if keys.is_empty() {
                panic!("keys not set");
            }

            if let Some(period) = period {
                loop {
                    let results = app
                        .force_price_feed_timestamps(&keys)
                        .await
                        .expect("force timestamps");
                    results.into_iter().for_each(|view| match view {
                        PriceFeedSignatureView {
                            price_feed,
                            signature: Some(signature),
                            error: None,
                        } => info!(%price_feed, %signature, "price-feed timestamp updated"),
                        PriceFeedSignatureView {
                            price_feed: key,
                            error: Some(err),
                            ..
                        } => {
                            warn!(%key, %err, "update price-feed timestamp");
                        }
                        _ => unreachable!(),
                    });
                    tokio::time::sleep(period).await;
                }
            } else {
                let results = app
                    .force_price_feed_timestamps(&keys)
                    .await
                    .expect("force timestamps");
                println_cmd_out!(&results);
            }
        }
        opts::Command::GetFeedIdFromHex { hex } => {
            let feed_id = get_feed_id_from_hex(hex.as_str()).expect("feed id from hex");
            let feed_pubkey = Pubkey::from(feed_id);
            println!("{}", feed_pubkey)
        }

        opts::Command::UpdatePrice {
            key,
            maximum_age_sec,
            pyth_api_url,
        } => {
            let client = PriceProxyClient {
                rpc: RpcClient::new_with_commitment(
                    opts.url.clone(),
                    CommitmentConfig {
                        commitment: opts.commitment,
                    },
                ),
                authority: app.authority,
                priority_fee: app.priority_fee,
                show_spinner: true,
            };

            let signatures = client
                .holistic_update_price(&key, maximum_age_sec, pyth_api_url)
                .await
                .map_err(|err| println!("Error updating price feed: {}", err))
                .unwrap();

            for sig in signatures {
                println_cmd_out!(&sig);
            }
        }
        opts::Command::AlterPriceFeed {
            key,
            feed_type,
            symbol,
            quote_symbol,
            logo_url,
            source,
            transform_source,
            source_address,
            transform_source_address,
            verification_level,
        } => {
            let signature = app
                .alter_price_feed(
                    key,
                    feed_type,
                    symbol,
                    quote_symbol,
                    verification_level,
                    logo_url,
                    source,
                    transform_source,
                    source_address,
                    transform_source_address,
                )
                .await
                .expect("altered Price-feed");

            println_cmd_out!(&signature);
            println!("Altered Price-feed: {}", key);
        }

        opts::Command::DeletePriceFeed { key } => {
            let signature = app.delete_price_feed(key).await.expect("delete Price-feed");

            println_cmd_out!(&signature);
            println!("Deleted Price-feed: {}", key);
        }
        opts::Command::ShowStakePoolPrice { key, symbol } => loop {
            let stakepool_acc = get_account_with_retries(&app.rpc, &key)
                .await
                .expect("get StakePool acc");
            let source_data: &[u8] = &stakepool_acc.data;
            let data_feed = try_from_slice_unchecked::<StakePool>(source_data).expect("unpack acc");

            let lst_price = Decimal::from_i128_with_scale(data_feed.total_lamports as i128, 0)
                .expect("from_i128")
                .checked_div(
                    Decimal::from_i128_with_scale(data_feed.pool_token_supply as i128, 0)
                        .expect("from_i128"),
                )
                .expect("checked_dev");

            info!("{} price: {}", symbol, lst_price);

            tokio::time::sleep(Duration::from_secs(5)).await;
        },
        opts::Command::ContractVersion {} => {
            app.contract_version().await;
        }
    }
}

macro_rules! println_cmd_out {
    ($out:expr) => {{
        let out = serde_json::to_string_pretty($out).expect("json");
        println!("{out}");
    }};
}
pub(crate) use println_cmd_out;

fn tracing_init() -> tracing_appender::non_blocking::WorkerGuard {
    use tracing_subscriber::filter::LevelFilter;
    use tracing_subscriber::fmt::Subscriber;
    use tracing_subscriber::util::SubscriberInitExt;

    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stderr());

    let builder = Subscriber::builder();
    let builder = builder
        .with_max_level(LevelFilter::TRACE)
        // .with_ansi(false)
        .with_writer(non_blocking);

    let subscriber = builder.finish();
    let subscriber = {
        use std::{env, str::FromStr};
        use tracing_subscriber::{filter::Targets, layer::SubscriberExt};
        let targets = match env::var("RUST_LOG") {
            Ok(var) => var,
            Err(_) => "warn".to_owned(),
        };
        subscriber.with(Targets::from_str(&targets).unwrap())
    };

    subscriber.try_init().expect("init tracing");

    guard
}
