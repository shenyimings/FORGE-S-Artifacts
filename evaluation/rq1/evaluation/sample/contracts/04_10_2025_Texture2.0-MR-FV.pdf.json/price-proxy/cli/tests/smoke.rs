use std::time::Duration;

use price_proxy::state::price_feed::{PriceFeed, PriceFeedParams};
use price_proxy_client::{PriceFeedSignatureView, PriceFeedView, PriceFeedsView};

pub mod utils;
use tracing::debug;
use utils::*;

#[tokio::test]
async fn create_price_feed() {
    let ctx = init_test().await;
    let payer_key = ctx.payer.pubkey();

    let mut expected_price_feed = PriceFeed::new(
        PriceFeedParams::new(
            "Direct",
            "SOL",
            "USD",
            "full",
            "/sol",
            "off-chain",
            "off-chain",
        ),
        payer_key,
        payer_key,
        payer_key,
    );
    let PriceFeedSignatureView {
        price_feed: price_feed_key,
        ..
    } = ctx
        .price_proxy_cmd_assert(&[
            "create-price-feed",
            "--feed-type",
            &expected_price_feed.feed_type().to_string(),
            "--symbol",
            &expected_price_feed.symbol(),
            "--quote-symbol",
            &expected_price_feed.quote_symbol().to_string(),
            "--logo-url",
            &expected_price_feed.logo_url(),
            "--source",
            &expected_price_feed.source().to_string(),
            "--source-address",
            &expected_price_feed.source_address.to_string(),
        ])
        .success()
        .json_output();

    let PriceFeedView { price_feed, .. } = ctx
        .price_proxy_cmd_assert(&["price-feed", &price_feed_key.to_string()])
        .success()
        .json_output();
    expected_price_feed.update_timestamp = price_feed.update_timestamp;
    assert_eq!(price_feed, expected_price_feed);

    let PriceFeedsView { price_feeds, .. } = ctx
        .price_proxy_cmd_assert(&["price-feeds"])
        .success()
        .json_output();
    assert!(!price_feeds.is_empty());
    let price_feed = price_feeds
        .get(&price_feed_key)
        .expect("get price-feed from all");
    assert_eq!(price_feed, &expected_price_feed);
}

#[tokio::test]
async fn write_price() {
    let ctx = init_test().await;
    let payer_key = ctx.payer.pubkey();

    let mut expected_price_feed = PriceFeed::new(
        PriceFeedParams::new(
            "Direct",
            "SOL",
            "USD",
            "full",
            "/sol",
            "off-chain",
            "off-chain",
        ),
        payer_key,
        payer_key,
        payer_key,
    );
    let PriceFeedSignatureView {
        price_feed: price_feed_key,
        ..
    } = ctx
        .price_proxy_cmd_assert(&[
            "create-price-feed",
            "--feed-type",
            &expected_price_feed.feed_type().to_string(),
            "--symbol",
            &expected_price_feed.symbol(),
            "--logo-url",
            &expected_price_feed.logo_url(),
            "--source",
            &expected_price_feed.source().to_string(),
            "--source-address",
            &expected_price_feed.source_address.to_string(),
        ])
        .success()
        .json_output();

    let new_price = dec!(1.02);
    let _ = ctx
        .price_proxy_cmd_assert(&[
            "write-price",
            &price_feed_key.to_string(),
            &new_price.to_string(),
        ])
        .success();

    let PriceFeedView { price_feed, .. } = ctx
        .price_proxy_cmd_assert(&["price-feed", &price_feed_key.to_string()])
        .success()
        .json_output();
    expected_price_feed
        .try_set_price(
            new_price,
            price_feed.update_timestamp,
            price_feed.update_slot,
        )
        .unwrap();
    assert_eq!(price_feed, expected_price_feed);
}

#[tokio::test]
async fn force_price_timestamps() {
    let ctx = init_test().await;
    let payer_key = ctx.payer.pubkey();

    let price_feed_keys: Vec<_> = (0..3)
        .map(|_| {
            let PriceFeedSignatureView { price_feed, .. } = ctx
                .price_proxy_cmd_assert(&[
                    "create-price-feed",
                    "--feed-type",
                    "direct",
                    "--symbol",
                    "SOL",
                    "--logo-url",
                    "/sol",
                    "--source",
                    "off-chain",
                    "--source-address",
                    &payer_key.to_string(),
                ])
                .success()
                .json_output();
            price_feed
        })
        .collect();

    let PriceFeedSignatureView {
        price_feed: invalid_src_key,
        ..
    } = ctx
        .price_proxy_cmd_assert(&[
            "create-price-feed",
            "--feed-type",
            "direct",
            "--symbol",
            "SOL",
            "--logo-url",
            "/sol",
            "--source",
            "Pyth",
            "--source-address",
            &Pubkey::new_unique().to_string(),
        ])
        .success()
        .json_output();

    let _ = ctx
        .price_proxy_cmd_assert(&[
            "force-price-timestamp",
            "--key",
            &price_feed_keys[0].to_string(),
            "--key",
            &invalid_src_key.to_string(),
            "--key",
            &price_feed_keys[1].to_string(),
        ])
        .success();

    let PriceFeedsView { price_feeds, .. } = ctx
        .price_proxy_cmd_assert(&[
            "price-feeds",
            "--key",
            &price_feed_keys[0].to_string(),
            "--key",
            &price_feed_keys[1].to_string(),
        ])
        .success()
        .json_output();
    assert_eq!(price_feeds.len(), 2);
    price_feeds.iter().for_each(|(key, feed)| {
        assert_eq!(feed.price_raw, 0, "{key} price changed");
        assert_ne!(feed.update_timestamp, 0, "{key} timestamp not updated");
    });
}

#[tokio::test]
async fn auto_force_price_timestamps() {
    let ctx = init_test().await;
    let payer_key = ctx.payer.pubkey();

    let price_feed_keys: Vec<_> = (0..3)
        .map(|_| {
            let PriceFeedSignatureView { price_feed, .. } = ctx
                .price_proxy_cmd_assert(&[
                    "create-price-feed",
                    "--feed-type",
                    "direct",
                    "--symbol",
                    "SOL",
                    "--logo-url",
                    "/sol",
                    "--source",
                    "off-chain",
                    "--source-address",
                    &payer_key.to_string(),
                ])
                .success()
                .json_output();
            price_feed
        })
        .collect();

    let period_sec = 1;
    let mut timestamp_refresher = ctx.price_proxy_cmd_spawn(&[
        "--commitment",
        "processed",
        "force-price-timestamp",
        "--period",
        &humantime::Duration::from(Duration::from_secs(period_sec)).to_string(),
        "--key",
        &price_feed_keys[0].to_string(),
        "--key",
        &price_feed_keys[1].to_string(),
    ]);

    let period = Duration::from_secs(period_sec + 2 /* secs for confirm transactions */);
    let mut prev_timestamp = 0;
    for _ in 0..3 {
        tokio::time::sleep(period).await;

        let PriceFeedsView { price_feeds, .. } = ctx
            .price_proxy_cmd_assert(&[
                "price-feeds",
                "--key",
                &price_feed_keys[0].to_string(),
                "--key",
                &price_feed_keys[1].to_string(),
            ])
            .success()
            .json_output();
        assert_eq!(price_feeds.len(), 2);

        let cur_timestamp = price_feeds
            .get(&price_feed_keys[0])
            .unwrap()
            .update_timestamp;
        debug!(%prev_timestamp, %cur_timestamp);

        price_feeds.iter().for_each(|(key, feed)| {
            assert_eq!(feed.price_raw, 0, "{key} price changed");
            assert_eq!(feed.update_timestamp, cur_timestamp);
            assert!(
                feed.update_timestamp > prev_timestamp,
                "{key} timestamp not updated"
            );
        });

        prev_timestamp = cur_timestamp;
    }

    timestamp_refresher.kill().await.unwrap();
}
