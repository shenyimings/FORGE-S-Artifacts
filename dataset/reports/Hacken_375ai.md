# Introduction

We express our gratitude to the 375ai team for the collaborative engagement that enabled the execution of this Smart Contract Security Assessment.

The Rewards Distributor enables users to claim rewards based on their contributions using Merkle trees for fair and efficient verification. This system ensures transparent and secure reward distribution by allowing users to submit proof of participation to claim their funds


# Review Scope

Repository	https://github.com/375-ai/program-library
Initial Commit	b1bf39bbbe62418c8ef63c59f537fbb25e245b0d
Re-audit Commit	d2fac818499300111f9058ebac44f335fefb7bfd

# Audit Findings

## F-2024-5204 Irrelevant and Redundant Limitations on maxtotalclaim and maxnumnodes

Status
fixed
Severity
Medium
Impact
5
Likelihood
2
Complexity
Simple
Exploitability
Independent
Description
The current implementation imposes limitations on max_total_claim and max_num_nodes, which introduces unnecessary complexity and potential issues in the reward distribution process. The max_num_nodes limitation seems irrelevant and does not contribute to the intended functionality of the system. Additionally, the max_total_claim limitation, which is intended to cap the total amount of tokens that can be claimed by all users, contradicts the business logic and can create a first-come, first-served scenario. This limitation is redundant because the amount of tokens that users can claim is inherently limited by the total number of tokens allocated to the RewardDistributor's account. As such, this invariant is unnecessary and does not enhance the security or functionality of the system.

Incorrect max_total_claim parameter can lead to locked tokens in the RewardDistributor account. Consider the following scenario:

Admin creates a new distribution for 1000 tokens. It is defined in a Merkle tree.

By mistake, the admin set max_total_claim as 100 tokens.

Admin transfers all expected 1000 tokens to RewardDistributor associated token account.

Users can claim only 100 tokens - max_total_claim

Other 900 tokens are not claimable and are locked in the  RewardDistributor associated token account.

Setting the parameters:

pub fn set_merkle_distributor_handler(
    ctx: Context<NewDistributor>,
    bump: u8,
    root: [u8; 32],
    max_total_claim: u64,
    max_num_nodes: u64,
) -> Result<()> {
    ...
    distributor.max_total_claim = max_total_claim;
    distributor.max_num_nodes = max_num_nodes;
Checks:

/// claim handler.
pub fn claim_handler(
    ctx: Context<Claim>,
    index: u64,
    amount: u64,
    proof: Vec<[u8; 32]>,
) -> Result<()> {
    ...
    // Update the distributor's total amount claimed and number of nodes claimed.
    let distributor = &mut ctx.accounts.distributor;
    distributor.total_amount_claimed = distributor.total_amount_claimed + amount;
    require!(
        distributor.total_amount_claimed <= distributor.max_total_claim,
        ErrorCode::ExceededMaxClaim
    );
    distributor.num_nodes_claimed += 1;
    require!(
        distributor.num_nodes_claimed <= distributor.max_num_nodes,
        ErrorCode::ExceededMaxNumNodes
    );
}

## F-2024-4202 Missing Two-Step Ownership Pattern for Admin Changes

Status
fixed
Severity
Low
Impact
4
Likelihood
1
Complexity
Simple
Exploitability
Independent
Description
The current implementation lacks a two-step ownership pattern when changing the admin of the RewardsDistributor. This omission poses a risk where the ownership could be transferred to an incorrect or invalid public key, potentially bricking the contract and making it inoperable. Such a situation would prevent any further administrative actions, including setting the Merkle root.

The update_admin instruction:

pub fn update_admin_handler(ctx: Context<UpdateAdmin>, new_admin: Pubkey) -> Result<()> {
    // Get a mutable reference to the rewards account from the context.
    let rewards_account = &mut ctx.accounts.rewards_account;

    // Update the admin field of the rewards account to the new admin.
    rewards_account.admin = new_admin;

    // Emit an event to signal that the admin has been updated.
    emit!(AdminUpdated { new_admin });

    Ok(())
}