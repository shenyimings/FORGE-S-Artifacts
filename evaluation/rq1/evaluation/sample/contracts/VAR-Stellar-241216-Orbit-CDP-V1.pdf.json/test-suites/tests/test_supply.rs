use test_suites::create_fixture_with_data;
use test_suites::test_fixture::TokenIndex;

#[test]
fn test_increase_supply() {
    let fixture = create_fixture_with_data(false, false);

    let pool_fixture = &fixture.pools[0];

    let ousd_balance = fixture.tokens[TokenIndex::OUSD].balance(&pool_fixture.pool.address);

    let amount = 100_000;
    fixture.admin_contract.update_supply(&fixture.tokens[TokenIndex::OUSD].address.clone(), &amount);

    assert_eq!(
        ousd_balance + amount,
        fixture.tokens[TokenIndex::OUSD].balance(&pool_fixture.pool.address)
    );
}

#[test]
fn test_decrease_supply() {
    let fixture = create_fixture_with_data(false, false);

    let pool_fixture = &fixture.pools[0];

    let ousd_balance = fixture.tokens[TokenIndex::OUSD].balance(&pool_fixture.pool.address);

    let amount = -100_000;
    fixture.admin_contract.update_supply(&fixture.tokens[TokenIndex::OUSD].address.clone(), &amount);

    assert_eq!(
        ousd_balance + amount,
        fixture.tokens[TokenIndex::OUSD].balance(&pool_fixture.pool.address)
    );
}

#[test]
#[should_panic = "Error(Contract, #1504)"]
fn test_not_enough_supply() {
    let fixture = create_fixture_with_data(false, false);

    let pool_fixture = &fixture.pools[0];

    let ousd_balance = fixture.tokens[TokenIndex::OUSD].balance(&pool_fixture.pool.address);

    let amount = -ousd_balance - 100_000_000_000;
    fixture.admin_contract.update_supply(&fixture.tokens[TokenIndex::OUSD].address.clone(), &amount);
}

#[test]
#[should_panic = "Error(Contract, #1502)"]
fn test_zero_amount() {
    let fixture = create_fixture_with_data(false, false);

    let amount = 0;
    fixture.admin_contract.update_supply(&fixture.tokens[TokenIndex::OUSD].address.clone(), &amount);
}