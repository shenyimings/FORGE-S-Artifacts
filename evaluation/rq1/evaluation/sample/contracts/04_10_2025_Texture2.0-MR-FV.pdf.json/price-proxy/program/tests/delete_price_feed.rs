use price_proxy_client::{PriceFeedSignatureView, SignatureView};

use pretty_assertions::assert_eq;
use price_proxy::instruction::DeletePriceFeed;
use tracing::info;

use price_proxy::state::price_feed::{PriceFeed, PriceFeedParams};

mod utils;
use utils::*;

#[tokio::test]
async fn delete_success() {
    let TestContext {
        price_proxy,
        payer: _,
    } = init_test().await;

    let price_feed_lamports = price_proxy
        .rpc
        .get_minimum_balance_for_rent_exemption(PriceFeed::SIZE)
        .await
        .unwrap();

    let authority_key = price_proxy.authority.pubkey();

    let params = PriceFeedParams::new(
        "Direct",
        "SOL",
        "USD",
        "full",
        "/sol",
        "switchboard",
        "switchboard",
    );

    // CREATE PRICE-FEED

    let PriceFeedSignatureView {
        price_feed: feed_key,
        ..
    } = price_proxy
        .create_price_feed(params, SB_SOL_PRICE_SOURCE, SB_SOL_PRICE_SOURCE)
        .await
        .expect("create price-feed");

    // DELETE PRICE-FEED

    info!("delete price-feed");

    let authority_acc0 = price_proxy
        .rpc
        .get_account(&authority_key)
        .await
        .expect("get authority acc");

    let SignatureView { .. } = price_proxy
        .delete_price_feed(feed_key)
        .await
        .expect("delete price feed");

    let authority_acc1 = price_proxy
        .rpc
        .get_account(&authority_key)
        .await
        .expect("get authority acc");

    let tx_fee = 5000;
    assert_eq!(
        authority_acc1.lamports,
        authority_acc0.lamports + price_feed_lamports - tx_fee
    );
    price_proxy
        .rpc
        .get_account(&feed_key)
        .await
        .expect_err("get price-feed acc");
}

#[tokio::test]
async fn delete_incorrect_authority() {
    let TestContext {
        price_proxy,
        payer: other_authority,
    } = init_test().await;
    let other_authority_pubkey = other_authority.pubkey();

    let params = PriceFeedParams::new(
        "Direct",
        "SOL",
        "USD",
        "full",
        "/sol",
        "switchboard",
        "switchboard",
    );

    // CREATE PRICE-FEED

    let PriceFeedSignatureView {
        price_feed: feed_key,
        ..
    } = price_proxy
        .create_price_feed(params, SB_SOL_PRICE_SOURCE, SB_SOL_PRICE_SOURCE)
        .await
        .expect("create price-feed");

    // DELETE PRICE-FEED

    info!("delete price-feed with incorrect authority");

    let ix = DeletePriceFeed {
        price_feed: feed_key,
        authority: other_authority_pubkey,
    }
    .into_instruction();

    let result = price_proxy
        .send_transaction_by(vec![ix], &[&other_authority, &price_proxy.authority])
        .await;
    info!("{:#?}", result);
    assert!(result.is_err())
}
