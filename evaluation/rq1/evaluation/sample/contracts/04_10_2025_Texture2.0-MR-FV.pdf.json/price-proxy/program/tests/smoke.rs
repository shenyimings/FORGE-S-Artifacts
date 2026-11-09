use chrono::Utc;
use futures::future::join_all;
use price_proxy_client::{PriceFeedSignatureView, PriceFeedView, PriceFeedsView, SignatureView};

use pretty_assertions::assert_eq;
use tracing::info;

use price_proxy::state::price_feed::{PriceFeed, PriceFeedParams};

mod utils;
use utils::*;

#[tokio::test]
async fn offchain() {
    let TestContext {
        price_proxy,
        payer: _,
    } = init_test().await;
    let authority_key = price_proxy.authority.pubkey();

    let expected_params = PriceFeedParams::new(
        "Direct",
        "SOL",
        "USD",
        "full",
        "/sol",
        "off-chain",
        "off-chain",
    );
    let mut expected = PriceFeed::new(expected_params, authority_key, authority_key, authority_key);

    let PriceFeedSignatureView {
        price_feed: feed_key,
        ..
    } = price_proxy
        .create_price_feed(expected_params, authority_key, authority_key)
        .await
        .expect("create price-feed");

    let PriceFeedView { price_feed, .. } =
        price_proxy.price_feed(&feed_key).await.expect("price-feed");
    info!(%feed_key, %price_feed);
    assert_eq!(price_feed, expected);

    let new_price = dec!(1.05);
    let SignatureView { signature: _ } = price_proxy
        .write_price(feed_key, new_price, Utc::now().timestamp())
        .await
        .expect("write price");

    let PriceFeedView { price_feed, .. } =
        price_proxy.price_feed(&feed_key).await.expect("price-feed");
    info!(%feed_key, %price_feed);
    expected
        .try_set_price(
            new_price,
            price_feed.update_timestamp,
            price_feed.update_slot,
        )
        .unwrap();
    assert_eq!(price_feed, expected);
}

#[tokio::test]
async fn offchain_force_timestamps() {
    let TestContext {
        price_proxy,
        payer: _,
    } = init_test().await;
    let authority_key = price_proxy.authority.pubkey();

    let expected_params = PriceFeedParams::new(
        "Direct",
        "SOL",
        "USD",
        "full",
        "/sol",
        "off-chain",
        "off-chain",
    );
    let feed_keys = join_all((0..3).map(|_| async {
        let PriceFeedSignatureView { price_feed, .. } = price_proxy
            .create_price_feed(expected_params, authority_key, authority_key)
            .await
            .expect("create price-feed");
        price_feed
    }))
    .await;

    let _ = price_proxy
        .force_price_feed_timestamps(&feed_keys)
        .await
        .expect("force timestamps");

    let price_feeds_view = price_proxy
        .price_feeds_by_key(&feed_keys)
        .await
        .expect("price-feeds");
    info!(%price_feeds_view);
    let PriceFeedsView { price_feeds, .. } = price_feeds_view;
    assert_eq!(price_feeds.len(), 3);
    price_feeds.iter().for_each(|(key, feed)| {
        assert_eq!(feed.price_raw, 0, "{key} price changed");
        assert_ne!(feed.update_timestamp, 0, "{key} timestamp not updated");
    });
}
