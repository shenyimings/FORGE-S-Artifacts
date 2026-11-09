#![cfg(test)]
use soroban_sdk::{testutils::{Address as AddressTestTrait}, vec, Address, IntoVal, Symbol, Val, Vec};
use test_suites::{
    dependencies::pool::{Request, RequestType},
    create_fixture_with_data,
    test_fixture::{TokenIndex, SCALAR_7},
};

#[test]
fn test_liquidations_mock() {
    let mut fixture = create_fixture_with_data(true, false);

    let initial_xlm_amount = 10_000_000_000_00 * SCALAR_7; // Assuming 1 XLM
    let initial_ousd_amount = (initial_xlm_amount as f64 * 0.088) as i128;
    fixture.create_pair(TokenIndex::OUSD, TokenIndex::XLM, initial_ousd_amount, initial_xlm_amount);

    let pool_fixture = &fixture.pools[0];
    let henk = Address::generate(&fixture.env);
    let treasury = &fixture.mock_treasury;

    fixture.tokens[TokenIndex::XLM].mint(&henk, &(120_000 * SCALAR_7));

    let requests: Vec<Request> = vec![
        &fixture.env,
        Request {
            request_type: RequestType::SupplyCollateral as u32,
            address: fixture.tokens[TokenIndex::XLM].address.clone(),
            amount: 100_000 * SCALAR_7,
        },
        Request {
            request_type: RequestType::Borrow as u32,
            address: fixture.tokens[TokenIndex::OUSD].address.clone(),
            amount: 7_000 * SCALAR_7,
        },
    ];
    pool_fixture.pool.submit(&henk, &henk, &henk, &requests);

    assert_eq!(
        20_000 * SCALAR_7,
        fixture.tokens[TokenIndex::XLM].balance(&henk)
    );
    assert_eq!(
        7_000 * SCALAR_7,
        fixture.tokens[TokenIndex::OUSD].balance(&henk)
    );

    fixture.jump(60 * 60 * 24 * 7 * 4); // 4 weeks
    fixture.oracle.set_price_stable(&vec![
        &fixture.env,
        1_0000000,    // usdc
        0_0740000,    // xlm
        1_0000000,    // USD
        1_1000000,    // EURO
        1_2000000,    // GBP
    ]);

    // Create the token pair with initial supply.

    let pegkeeper = &fixture.mock_pegkeeper;

    let liq_pct = 100;
    let auction_data = pool_fixture
        .pool
        .new_liquidation_auction(&henk, &liq_pct);

    let ousd_bid_amount = auction_data.bid.get_unchecked(fixture.tokens[TokenIndex::OUSD].address.clone());
    let xlm_lot_amount = auction_data.lot.get_unchecked(fixture.tokens[TokenIndex::XLM].address.clone());

    //allow 250 blocks to pass
    fixture.jump_with_sequence(251 * 5);

    let pair = &fixture.pairs[0];

    let token = fixture.tokens[TokenIndex::OUSD].address.clone();
    let args: Vec<Val> = vec![
        &fixture.env,
        token.into_val(&fixture.env),
        ousd_bid_amount.into_val(&fixture.env),
    ];
    let fl_receive_sym = Symbol::new(&fixture.env, "fl_receive");

    treasury.keep_peg(&fl_receive_sym, &args.clone());

    std::println!("OUSD Balance: {}", fixture.tokens[TokenIndex::OUSD].balance(&pegkeeper.address.clone()) / SCALAR_7);
    std::println!("XLM Balance: {}", fixture.tokens[TokenIndex::XLM].balance(&pegkeeper.address.clone()));

    pegkeeper.liquidate(&henk, &fixture.tokens[TokenIndex::OUSD].address.clone(), &ousd_bid_amount, &fixture.tokens[TokenIndex::XLM].address.clone(), &xlm_lot_amount, &pool_fixture.pool.address.clone(), &(100 as i128));

    std::println!("OUSD Balance: {}", fixture.tokens[TokenIndex::OUSD].balance(&pegkeeper.address.clone()) / SCALAR_7);
    std::println!("XLM Balance: {}", fixture.tokens[TokenIndex::XLM].balance(&pegkeeper.address.clone()));

    pegkeeper.swap(&pair.address.clone(), &fixture.tokens[TokenIndex::OUSD].address.clone(), &fixture.tokens[TokenIndex::XLM].address.clone(), &xlm_lot_amount, &0);

    std::println!("OUSD Balance: {}", fixture.tokens[TokenIndex::OUSD].balance(&pegkeeper.address.clone()) / SCALAR_7);
    std::println!("XLM Balance: {}", fixture.tokens[TokenIndex::XLM].balance(&pegkeeper.address.clone()));
}

#[test]
fn test_pegkeeper() {
    let mut fixture = create_fixture_with_data(false, false);

    let initial_xlm_amount = 10_000_000_000_00 * SCALAR_7;
    let initial_ousd_amount = (initial_xlm_amount as f64 * 0.088) as i128;
    fixture.create_pair(TokenIndex::OUSD, TokenIndex::XLM, initial_ousd_amount, initial_xlm_amount);

    let pool_fixture = &fixture.pools[0];
    let henk = Address::generate(&fixture.env);
    let treasury = &fixture.treasury;

    fixture.tokens[TokenIndex::XLM].mint(&henk, &(120_000 * SCALAR_7));

    let requests: Vec<Request> = vec![
        &fixture.env,
        Request {
            request_type: RequestType::SupplyCollateral as u32,
            address: fixture.tokens[TokenIndex::XLM].address.clone(),
            amount: 100_000 * SCALAR_7,
        },
        Request {
            request_type: RequestType::Borrow as u32,
            address: fixture.tokens[TokenIndex::OUSD].address.clone(),
            amount: 7_000 * SCALAR_7,
        },
    ];
    pool_fixture.pool.submit(&henk, &henk, &henk, &requests);

    assert_eq!(
        20_000 * SCALAR_7,
        fixture.tokens[TokenIndex::XLM].balance(&henk)
    );
    assert_eq!(
        7_000 * SCALAR_7,
        fixture.tokens[TokenIndex::OUSD].balance(&henk)
    );

    fixture.jump(60 * 60 * 24 * 7 * 4); // 4 weeks
    fixture.oracle.set_price_stable(&vec![
        &fixture.env,
        1_0000000,    // usdc
        0_0740000,    // xlm
        1_0000000,    // USD
        1_1000000,    // EURO
        1_2000000,    // GBP
    ]);
    let liq_pct = 100;
    let auction_data = pool_fixture
        .pool
        .new_liquidation_auction(&henk, &liq_pct);

    let token = fixture.tokens[TokenIndex::OUSD].address.clone();
    let amount = auction_data.bid.get_unchecked(fixture.tokens[TokenIndex::OUSD].address.clone());
    let blend_pool = pool_fixture.pool.address.clone();
    let auction = henk.clone();
    let collateral_token = fixture.tokens[TokenIndex::XLM].address.clone();
    let lot_amount = auction_data.lot.get_unchecked(collateral_token.clone());
    let amm = fixture.pairs[0].address.clone();

    //allow 250 blocks to pass
    fixture.jump_with_sequence(250 * 5);

    let piet = Address::generate(&fixture.env);
    let args: Vec<Val> = vec![
        &fixture.env,
        token.into_val(&fixture.env),
        amount.into_val(&fixture.env),
        blend_pool.into_val(&fixture.env),
        auction.into_val(&fixture.env),
        collateral_token.into_val(&fixture.env),
        lot_amount.into_val(&fixture.env),
        (liq_pct as i128).into_val(&fixture.env),
        amm.into_val(&fixture.env),
        piet.into_val(&fixture.env),
    ];
    let fl_receive_sym = Symbol::new(&fixture.env, "fl_receive");

    treasury.keep_peg(&fl_receive_sym, &args.clone());

    let pegkeeper = &fixture.pegkeeper;
    let pegkeeper_xlm = &fixture.tokens[TokenIndex::XLM].balance(&pegkeeper.address.clone());
    let pegkeeper_ousd = &fixture.tokens[TokenIndex::OUSD].balance(&pegkeeper.address.clone());

    let piet_o = &fixture.tokens[TokenIndex::OUSD].balance(&piet);

    assert_eq!(pegkeeper_xlm, &0);
    assert_eq!(pegkeeper_ousd, &0);
    assert!(piet_o > &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #1505)")]
fn test_pegkeeper_no_profit() {
    let mut fixture = create_fixture_with_data(false, false);

    let initial_xlm_amount = 10_000_000_000_00 * SCALAR_7;
    let initial_ousd_amount = (initial_xlm_amount as f64 * 0.05) as i128; // Lower price a lot so that the liquidation is not profitable
    fixture.create_pair(TokenIndex::OUSD, TokenIndex::XLM, initial_ousd_amount, initial_xlm_amount);

    let pool_fixture = &fixture.pools[0];
    let henk = Address::generate(&fixture.env);
    let treasury = &fixture.treasury;

    fixture.tokens[TokenIndex::XLM].mint(&henk, &(120_000 * SCALAR_7));

    let requests: Vec<Request> = vec![
        &fixture.env,
        Request {
            request_type: RequestType::SupplyCollateral as u32,
            address: fixture.tokens[TokenIndex::XLM].address.clone(),
            amount: 100_000 * SCALAR_7,
        },
        Request {
            request_type: RequestType::Borrow as u32,
            address: fixture.tokens[TokenIndex::OUSD].address.clone(),
            amount: 7_000 * SCALAR_7,
        },
    ];
    pool_fixture.pool.submit(&henk, &henk, &henk, &requests);

    assert_eq!(
        20_000 * SCALAR_7,
        fixture.tokens[TokenIndex::XLM].balance(&henk)
    );
    assert_eq!(
        7_000 * SCALAR_7,
        fixture.tokens[TokenIndex::OUSD].balance(&henk)
    );

    fixture.jump(60 * 60 * 24 * 7 * 4); // 4 weeks
    fixture.oracle.set_price_stable(&vec![
        &fixture.env,
        1_0000000,    // usdc
        0_0740000,    // xlm
        1_0000000,    // USD
        1_1000000,    // EURO
        1_2000000,    // GBP
    ]);
    let liq_pct = 100;
    let auction_data = pool_fixture
        .pool
        .new_liquidation_auction(&henk, &liq_pct);

    let token = fixture.tokens[TokenIndex::OUSD].address.clone();
    let amount = auction_data.bid.get_unchecked(fixture.tokens[TokenIndex::OUSD].address.clone());
    let blend_pool = pool_fixture.pool.address.clone();
    let auction = henk.clone();
    let collateral_token = fixture.tokens[TokenIndex::XLM].address.clone();
    let lot_amount = auction_data.lot.get_unchecked(collateral_token.clone());
    let amm = fixture.pairs[0].address.clone();

    //allow 250 blocks to pass
    fixture.jump_with_sequence(250 * 5);

    let piet = Address::generate(&fixture.env);
    let args: Vec<Val> = vec![
        &fixture.env,
        token.into_val(&fixture.env),
        amount.into_val(&fixture.env),
        blend_pool.into_val(&fixture.env),
        auction.into_val(&fixture.env),
        collateral_token.into_val(&fixture.env),
        lot_amount.into_val(&fixture.env),
        (liq_pct as i128).into_val(&fixture.env),
        amm.into_val(&fixture.env),
        piet.into_val(&fixture.env),
    ];
    let fl_receive_sym = Symbol::new(&fixture.env, "fl_receive");

    treasury.keep_peg(&fl_receive_sym, &args.clone());
}



