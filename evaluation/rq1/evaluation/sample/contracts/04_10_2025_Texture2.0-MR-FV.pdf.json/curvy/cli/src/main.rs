use anyhow::anyhow;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::read_keypair_file;
use structopt::StructOpt;
use texture_common::_export::Zeroable;
use texture_common::math::Decimal;

use curvy::state::curve::{Curve, MAX_Y_CNT};
use curvy::state::curve::{CurveParams, CurveX, CurveY};
use curvy_client::CurvyClient as App;
use curvy_utils::calc_y;

mod opts;

#[derive(serde::Deserialize)]
struct Row {
    x: CurveX,
    #[serde(deserialize_with = "curve_y_from_string")]
    f_x: CurveY,
}

fn curve_y_from_string<'de, D>(deserializer: D) -> anyhow::Result<CurveY, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    s.replace('.', "")
        .parse::<CurveY>()
        .map_err(D::Error::custom)
}

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
    };

    match opts.cmd {
        opts::Command::CreateCurve {
            name,
            formula,
            decimals,
            csv,
        } => {
            let points_list = csv::Reader::from_path(csv)
                .expect("read csv file")
                .records()
                .map(|record| {
                    let row = record
                        .expect("parse csv file")
                        .deserialize::<Row>(None)
                        .expect("deserialize csv row");
                    (row.x as CurveX, row.f_x as CurveY)
                })
                .collect::<Vec<_>>();

            let mut y_values: [CurveY; MAX_Y_CNT] = Zeroable::zeroed();

            for (i, (_x, y)) in points_list.iter().enumerate() {
                if i >= MAX_Y_CNT {
                    println!("Error: max {} points allowed", MAX_Y_CNT);
                }
                y_values[i] = *y;
            }

            let params = CurveParams::new(
                &name,
                &formula,
                points_list[0].0,
                points_list[1].0 - points_list[0].0,
                points_list.len() as u8,
                decimals,
                y_values,
            );
            let created_curve = app
                .create_curve(params, app.priority_fee)
                .await
                .expect("create curve");
            println_cmd_out!(&created_curve);
        }
        opts::Command::AlterCurve {
            curve,
            name,
            formula,
            decimals,
            csv,
        } => {
            let (x0, x_step, y_count, y) = if let Some(csv) = csv {
                let points_list = csv::Reader::from_path(csv)
                    .expect("read csv file")
                    .records()
                    .map(|record| {
                        let row = record
                            .expect("parse csv file")
                            .deserialize::<Row>(None)
                            .expect("deserialize csv row");
                        (row.x as CurveX, row.f_x as CurveY)
                    })
                    .collect::<Vec<_>>();

                let mut y_values: [CurveY; MAX_Y_CNT] = Zeroable::zeroed();

                for (i, (_x, y)) in points_list.iter().enumerate() {
                    if i >= MAX_Y_CNT {
                        println!("Error: max {} points allowed", MAX_Y_CNT);
                    }
                    y_values[i] = *y;
                }

                (
                    Some(points_list[0].0),
                    Some(points_list[1].0 - points_list[0].0),
                    Some(points_list.len() as u8),
                    Some(y_values),
                )
            } else {
                (None, None, None, None)
            };

            let signature = app
                .alter_curve(
                    curve,
                    name,
                    formula,
                    decimals,
                    x0,
                    x_step,
                    y_count,
                    y,
                    app.priority_fee,
                )
                .await
                .expect("alter curve");

            println!("{:#?}", signature);
            println!("altered curve: {}", curve);
        }
        opts::Command::DeleteCurve { curve } => {
            let signature = app
                .delete_curve(curve, app.priority_fee)
                .await
                .expect("delete curve");

            println!("{:#?}", signature);
            println!("deleted curve: {}", curve);
        }
        opts::Command::Curve { curve } => {
            let curve = app.curve(&curve).await.expect("get curve");
            println!("{}", curve);
        }
        opts::Command::Curves => {
            let curves = app.curves().await.expect("get curves");

            for curve in curves.curves {
                println!("{}", curve);

                print_x_y(&curve.curve);

                println!("======================================");
            }
        }
        opts::Command::CalcY { curve, x } => {
            let curve = app.curve(&curve).await.expect("get curve");

            let decimal_x =
                Decimal::from_i128_with_scale((x * 1_000_000_000.0) as i128, 9).unwrap();

            let y = calc_y(decimal_x, &curve.curve)
                .map_err(|err| println!("error: {}", err))
                .unwrap();

            println!("y = {}", y);
        }
    }
}

pub fn print_x_y(curve: &Curve) {
    println!("  X  :  f(x)");
    let mut x = curve.x0;
    for idx in 0..curve.y_count {
        println!(
            "  {}  :  {}",
            x as f32 / 10_u32.pow(curve.decimals as u32) as f32,
            curve.y[idx as usize] as f32 / 10_u32.pow(curve.decimals as u32) as f32
        );
        x += curve.x_step;
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
