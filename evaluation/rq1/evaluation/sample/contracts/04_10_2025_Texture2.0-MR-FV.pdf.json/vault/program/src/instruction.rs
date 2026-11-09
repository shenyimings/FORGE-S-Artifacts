use crate::state::vault::VaultParams;
use crate::VAULT_ID;
use crate::{pda::AUTHORITY_SEED, state::reward::RewardRules};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use super_lendy::SUPER_LENDY_ID;

#[derive(texture_common::macros::Instruction, BorshSerialize, BorshDeserialize)]
#[instruction(out_dir = "src/instruction", out_mod = "generated", program_id = VAULT_ID, docs_module = ix_docs)]
pub enum VaultInstruction {
    /// Used to either print contract version or to allocate more compute budget for TX
    #[doc = ix_docs::version!()]
    Version { print_version: bool },
    /// Create Vault account
    #[doc = ix_docs::create_vault!()]
    #[accounts(
        account(
            name = "vault",
            docs = ["Vault account to create."],
            flags(writable, signer),
            checks(owner = "system", size = 0),
        ),
        account(
            name = "curator_vaults_authority",
            docs = ["Authority assigned by Curator to manage Vaults."],
            flags(writable, signer),
        ),
        account(
            name = "curator",
            docs = ["Curator of the Vault."],
            checks(owner = SUPER_LENDY_ID),
        ),
        account(
            name = "program_authority",
            pda_seeds = [ AUTHORITY_SEED ],
        ),
        account(
            docs = ["Liquidity tokens mint"],
            name = "liquidity_mint",
        ),
        account(
            name = "liquidity_buffer",
            flags(writable),
        ),
        program(
            docs = ["SPL Token program - classic one"],
            name = "lp_token_program",
            id = spl_token::ID,
        ),
        program(
            docs = ["SPL Token program to manage liquidity tokens. Either classic or 2022"],
            name = "liquidity_token_program",
        ),
        program(id = "system", docs = ["System Program"]),
        list(
            name = "reserves",
            checks(owner = SUPER_LENDY_ID),
            count_field = reserves_count,
        ),
        list(
            name = "lp_token_mints",
            checks(owner = spl_token::ID),
            count_field = lp_token_mints_count,
        ),
        list(
            name = "lp_token_storages",
            flags(writable),
            checks(owner = "system"),
            count_field = lp_token_storages_count,
        ),
    )]
    CreateVault {
        #[instruction_builder(internal)]
        reserves_count: u8,
        #[instruction_builder(internal)]
        lp_token_mints_count: u8,
        #[instruction_builder(internal)]
        lp_token_storages_count: u8,
        params: VaultParams,
    },
    /// Change existing Vault. Fields `name`, `description`, `distribution_manager` can only be changed
    /// by curator_vaults_authority.
    /// Field `distribution` can be changed by either `curator_vaults_authority` or `distribution_manager`.
    /// Only percentage of the `distribution` can be changed via this IX.
    /// Mint of the Vault can not be changed at all.
    #[doc = ix_docs::alter_vault!()]
    #[accounts(
        account(
            name = "vault",
            docs = ["Vault account to change."],
            flags(writable),
            checks(owner = "self"),
        ),
        account(
            name = "curator_vaults_authority",
            docs = ["Authority assigned by Curator to manage Vaults."],
            flags(writable),
        ),
        account(
            name = "distribution_manager",
            docs = ["Authority who can change percentage in distribution."],
            flags(writable),
        ),
        account(
            name = "curator",
            docs = ["Curator of the Vault."],
            checks(owner = SUPER_LENDY_ID),
        )
    )]
    AlterVault { params: VaultParams },
    /// Propose new distribution item (new Reserve).
    #[doc = ix_docs::propose_distribution_item!()]
    #[accounts(
        account(
            name = "vault",
            docs = ["Vault account to change."],
            flags(writable),
            checks(owner = "self"),
        ),
        account(
            name = "curator_vaults_authority",
            docs = ["Authority assigned by Curator to manage Vaults."],
            flags(writable, signer),
        ),
        account(
            name = "curator",
            docs = ["Curator of the Vault."],
            checks(owner = SUPER_LENDY_ID),
        ),
        account(
            name = "reserve",
            docs = ["Reserve to be added in to distribution"],
            checks(owner = SUPER_LENDY_ID),
        ),
    )]
    ProposeDistributionItem { index: u8 },
    /// Adds previously proposed distribution item with 0 share in to the Vault. Share can be adjusted thereafter.
    /// Caller must choose which proposal to add in to effective Vault's distribution via `index` parameter.
    #[doc = ix_docs::add_distribution_item!()]
    #[accounts(
        account(
            name = "vault",
            docs = ["Vault account to change."],
            flags(writable),
            checks(owner = "self"),
        ),
        account(
            name = "curator_vaults_authority",
            docs = ["Authority assigned by Curator to manage Vaults."],
            flags(writable, signer),
        ),
        account(
            name = "curator",
            docs = ["Curator of the Vault."],
            checks(owner = SUPER_LENDY_ID),
        ),
        account(
            name = "lp_token_mint",
            docs = ["Mint of the reserve's LP tokens"],
            checks(owner = spl_token::ID),
        ),
        account(
            name = "lp_token_storage",
            docs = ["LP token storage to be used by Vault. Unitialized."],
            flags(writable),
            checks(owner = "system"),
        ),
        account(
            name = "program_authority",
            pda_seeds = [ AUTHORITY_SEED ],
        ),
        program(name = "token_program", id = spl_token::ID, docs = ["SPL Token Program"]),
        program(id = "system", docs = ["System Program"]),
    )]
    AddDistributionItem { index: u8 },
    /// Remove existing distribution item from the Vault. Item must have 0 associated LP tokens and 0 share.
    #[doc = ix_docs::remove_distribution_item!()]
    #[accounts(
        account(
            name = "vault",
            docs = ["Vault account to change."],
            flags(writable),
            checks(owner = "self"),
        ),
        account(
            name = "curator_vaults_authority",
            docs = ["Authority assigned by Curator to manage Vaults."],
            flags(writable, signer),
        ),
        account(
            name = "curator",
            docs = ["Curator of the Vault."],
            checks(owner = SUPER_LENDY_ID),
        ),
        account(
            name = "lp_token_storage",
            docs = ["LP token storage."],
            flags(writable),
            checks(owner = spl_token::ID),
        ),
        account(
            name = "program_authority",
            pda_seeds = [ AUTHORITY_SEED ],
        ),
        program(name = "token_program", id = spl_token::ID, docs = ["SPL Token Program"]),
    )]
    RemoveDistributionItem { reserve: Pubkey },
    /// Create Position
    #[doc = ix_docs::create_position!()]
    #[accounts(
        account(
            name = "position",
            flags(writable, signer),
            docs = ["Position account to create."],
            checks(owner = "system", size = 0),
        ),
        account(
            name = "vault",
            docs = ["Vault account the position belongs to."],
            checks(owner = "self"),
        ),
        account(
            name = "owner",
            docs = ["Position owner."],
            flags(writable, signer),
        ),
        account(
            name = "program_authority",
            pda_seeds = [ AUTHORITY_SEED ],
        ),
        program(id = "system", docs = ["System Program"]),
    )]
    CreatePosition,
    /// Deposit
    #[doc = ix_docs::deposit!()]
    #[accounts(
        account(
            name = "vault",
            docs = ["Vault account to deposit to."],
            flags(writable),
            checks(owner = "self"),
        ),
        account(
            name = "position",
            flags(writable),
            docs = ["Position account to deposit thru."],
            checks(owner = "self"),
        ),
        account(
            name = "owner",
            docs = ["Position owner."],
            flags(writable, signer),
        ),
        account(
            docs = ["Liquidity tokens mint"],
            name = "liquidity_mint",
        ),
        account(
            name = "liquidity_source",
            flags(writable),
        ),
        account(
            name = "liquidity_buffer",
            flags(writable),
        ),
        account(
            name = "program_authority",
            pda_seeds = [ AUTHORITY_SEED ],
        ),
        account(
            name = "superlendy_program_authority",
        ),
        program(
            docs = ["SPL Token program - classic one"],
            name = "lp_token_program",
            id = spl_token::ID,
        ),
        program(
            docs = ["SPL Token program to manage liquidity tokens. Either classic or 2022"],
            name = "liquidity_token_program",
        ),
        program(name = "superlendy_program", id = super_lendy::SUPER_LENDY_ID, docs = ["Superlendy Program"]),
        list(
            name = "reserves",
            flags(writable),
            checks(owner = SUPER_LENDY_ID),
            count_field = reserves_count,
        ),
        list(
            name = "reserve_liquidity_supplies",
            flags(writable),
            count_field = reserve_liquidity_supplies_count,
        ),
        list(
            name = "lp_token_mints",
            flags(writable),
            checks(owner = spl_token::ID),
            count_field = lp_token_mints_count,
        ),
        list(
            name = "lp_token_storages",
            flags(writable),
            checks(owner = spl_token::ID),
            count_field = lp_token_storages_count,
        ),
    )]
    Deposit {
        #[instruction_builder(internal)]
        reserves_count: u8,
        #[instruction_builder(internal)]
        reserve_liquidity_supplies_count: u8,
        #[instruction_builder(internal)]
        lp_token_mints_count: u8,
        #[instruction_builder(internal)]
        lp_token_storages_count: u8,
        amount: u64,
    },
    /// Withdraw from Vault
    #[doc = ix_docs::withdraw!()]
    #[accounts(
        account(
            name = "vault",
            flags(writable),
            docs = ["Vault account to withdraw from."],
            checks(owner = "self"),
        ),
        account(
            name = "position",
            flags(writable),
            docs = ["Position account to withdraw from."],
            checks(owner = "self"),
        ),
        account(
            name = "owner",
            docs = ["Position owner."],
            flags(writable, signer),
        ),
        account(
            docs = ["Liquidity tokens mint"],
            name = "liquidity_mint",
        ),
        account(
            name = "liquidity_destination",
            flags(writable),
        ),
        account(
            name = "liquidity_buffer",
            flags(writable),
        ),
        account(
            name = "program_authority",
            pda_seeds = [ AUTHORITY_SEED ],
        ),
        account(
            name = "superlendy_program_authority",
        ),
        program(
            docs = ["SPL Token program - classic one"],
            name = "lp_token_program",
            id = spl_token::ID,
        ),
        program(
            docs = ["SPL Token program to manage liquidity tokens. Either classic or 2022"],
            name = "liquidity_token_program",
        ),
        program(name = "superlendy_program", id = super_lendy::SUPER_LENDY_ID, docs = ["Superlendy Program"]),
        list(
            name = "reserves",
            flags(writable),
            checks(owner = SUPER_LENDY_ID),
            count_field = reserves_count,
        ),
        list(
            name = "reserve_liquidity_supplies",
            flags(writable),
            count_field = reserve_liquidity_supplies_count,
        ),
        list(
            name = "lp_token_mints",
            flags(writable),
            checks(owner = spl_token::ID),
            count_field = lp_token_mints_count,
        ),
        list(
            name = "lp_token_storages",
            flags(writable),
            checks(owner = spl_token::ID),
            count_field = lp_token_storages_count,
        ),
    )]
    Withdraw {
        #[instruction_builder(internal)]
        reserves_count: u8,
        #[instruction_builder(internal)]
        reserve_liquidity_supplies_count: u8,
        #[instruction_builder(internal)]
        lp_token_mints_count: u8,
        #[instruction_builder(internal)]
        lp_token_storages_count: u8,
        /// Desired VLP amount to redeem
        vlp_amount: u64,
    },
    /// Close Position
    #[doc = ix_docs::close_position!()]
    #[accounts(
        account(
            name = "position",
            flags(writable),
            docs = ["Position account to close."],
            checks(owner = "self"),
        ),
        account(
            name = "vault",
            checks(owner = "self"),
        ),
        account(
            docs = ["Owner of the position"],
            name = "owner",
            flags(writable, signer),
        ),
    )]
    ClosePosition,
    /// Rebalance Vault
    #[doc = ix_docs::rebalance!()]
    #[accounts(
        account(
            name = "vault",
            checks(owner = "self"),
        ),
        account(
            name = "program_authority",
            pda_seeds = [ AUTHORITY_SEED ],
        ),
        account(
            name = "liquidity_mint",
            docs = ["Liquidity tokens mint"],
        ),
        account(
            name = "liquidity_buffer",
            flags(writable),
        ),
        account(
            name = "superlendy_program_authority",
        ),
        program(
            docs = ["SPL Token program - classic one"],
            name = "lp_token_program",
            id = spl_token::ID,
        ),
        program(
            docs = ["SPL Token program to manage liquidity tokens. Either classic or 2022"],
            name = "liquidity_token_program",
        ),
        program(name = "superlendy_program", id = super_lendy::SUPER_LENDY_ID, docs = ["Superlendy Program"]),
        list(
            name = "reserves",
            flags(writable),
            checks(owner = SUPER_LENDY_ID),
            count_field = reserves_count,
        ),
        list(
            name = "reserve_liquidity_supplies",
            flags(writable),
            count_field = reserve_liquidity_supplies_count,
        ),
        list(
            name = "lp_token_mints",
            flags(writable),
            checks(owner = spl_token::ID),
            count_field = lp_token_mints_count,
        ),
        list(
            name = "lp_token_storages",
            flags(writable),
            checks(owner = spl_token::ID),
            count_field = lp_token_storages_count,
        ),
    )]
    Rebalance {
        #[instruction_builder(internal)]
        reserves_count: u8,
        #[instruction_builder(internal)]
        reserve_liquidity_supplies_count: u8,
        #[instruction_builder(internal)]
        lp_token_mints_count: u8,
        #[instruction_builder(internal)]
        lp_token_storages_count: u8,
    },
    /// Initialize contract controlled SPL wallet for particular reward token.
    #[doc = ix_docs::init_reward_supply!()]
    #[accounts(
        account(
            docs = ["Reward supply account to initialize. Uninitialized. PDA"],
            name = "reward_supply",
            flags(writable),
            checks(owner = "system"),
            pda_seeds = [vault, reward_mint, crate::pda::REWARD_SUPPLY_SEED],
        ),
        account(
            docs = ["Reward token mint."],
            name = "reward_mint",
        ),
        account(
            docs = ["Vault to init reward supply for."],
            name = "vault",
            checks(owner = "self"),
        ),
        account(
            docs = [
            "Authority who can manage Vault."
            ],
            name = "curator_vaults_authority",
            flags(writable, signer),
        ),
        account(
            docs = ["Curator account."],
            name = "curator",
            checks(owner = SUPER_LENDY_ID, exempt),
        ),
        account(
            docs = ["Contract's reward authority. PDA."],
            name = "reward_authority",
            pda_seeds = [vault, crate::pda::AUTHORITY_SEED],
        ),
        program(
            docs = ["SPL Token program"],
            name = "token_program",
        ),
        program(
            docs = ["System Program."],
            id = "system",
        ),
    )]
    InitRewardSupply,
    /// Set/change reward rule in the existing reserve.
    #[doc = ix_docs::set_reward_rules!()]
    #[accounts(
        account(
            docs = ["Vault account to set reward rules for"],
            name = "vault",
            flags(writable),
            checks(owner = "self"),
        ),
        account(
            docs = [
            "Authority who can configure Vault."
            ],
            name = "curator_vaults_authority",
            flags(writable, signer),
        ),
        account(
            docs = ["Curator account."],
            name = "curator",
            checks(owner = SUPER_LENDY_ID, exempt),
        ),
    )]
    SetRewardRules { rules: RewardRules },
    /// Claim reward tokens.
    #[doc = ix_docs::claim_reward!()]
    #[accounts(
        account(
            docs = ["Position account to claim rewords for. Refreshed."],
            name = "position",
            flags(writable),
            checks(owner = "self"),
        ),
        account(
            docs = ["Contract managed SPL token wallet which holds reward tokens. PDA."],
            name = "rewards_supply",
            flags(writable),
            pda_seeds = [vault, reward_mint, crate::pda::REWARD_SUPPLY_SEED],
        ),
        account(
            docs = ["User's SPL token wallet which will receive reward tokens"],
            name = "destination_wallet",
            flags(writable),
        ),
        account(
            docs = ["Position owner"],
            name = "position_owner",
            flags(signer),
        ),
        account(
            docs = ["Vault, position belongs to."],
            name = "vault",
            checks(owner = "self"),
        ),
        account(
            docs = ["Reward token mint to claim. Determines which reward will be claimed from the Position."],
            name = "reward_mint",
        ),
        account(
            docs = ["Contract's reward authority. PDA."],
            name = "reward_authority",
            pda_seeds = [vault, crate::pda::AUTHORITY_SEED],
        ),
        program(
            docs = ["SPL Token program"],
            name = "token_program",
        ),
    )]
    ClaimReward,
    /// Withdraw reward tokens from reward supply account
    /// Contract doesn't have DepositReward IX as it can be done by SplToken directly.
    #[doc = ix_docs::withdraw_reward!()]
    #[accounts(
        account(
            docs = ["Contract managed SPL token wallet which holds reward tokens. PDA."],
            name = "rewards_supply",
            flags(writable),
            pda_seeds = [vault, reward_mint, crate::pda::REWARD_SUPPLY_SEED],
        ),
        account(
            docs = ["SPL token account of reward_mint to receive reward tokens."],
            name = "destination_wallet",
            flags(writable),
        ),
        account(
            docs = ["Vault to withdraw rewards from."],
            name = "vault",
            checks(owner = "self"),
        ),
        account(
            docs = [
            "Authority who can configure Vault."
            ],
            name = "curator_vaults_authority",
            flags(writable, signer),
        ),
        account(
            docs = ["Curator account."],
            name = "curator",
            checks(owner = "self", exempt),
        ),
        account(
            docs = ["Reward token mint to withdraw."],
            name = "reward_mint",
        ),
        account(
            docs = ["Contract's reward authority. PDA."],
            name = "reward_authority",
            pda_seeds = [vault, crate::pda::AUTHORITY_SEED],
        ),
        program(
            docs = ["SPL Token program"],
            name = "token_program",
        ),
    )]
    WithdrawReward { amount: u64 },
    /// Claim accumulated fees
    #[doc = ix_docs::claim_fees!()]
    #[accounts(
        account(
            name = "vault",
            flags(writable),
            docs = ["Vault account to claim fees from."],
            checks(owner = "self"),
        ),
        account(
            name = "curator_vaults_authority",
            docs = ["Authority assigned by Curator to manage Vaults."],
            flags(signer),
        ),
        account(
            name = "curator",
            docs = ["Curator of the Vault."],
            checks(owner = SUPER_LENDY_ID),
        ),
        account(
            name = "liquidity_mint",
            docs = ["Liquidity tokens mint"],
        ),
        account(
            name = "fee_receiver",
            docs = ["Any SPL account of proper mint to transfer fees to"],
            flags(writable),
        ),
        account(
            name = "liquidity_buffer",
            docs = ["Any SPL account of proper mint to transfer fees to"],
            flags(writable),
        ),
        account(
            name = "program_authority",
            pda_seeds = [ AUTHORITY_SEED ],
        ),
        account(
            name = "superlendy_program_authority",
        ),
        program(
            docs = ["SPL Token program - classic one"],
            name = "lp_token_program",
            id = spl_token::ID,
        ),
        program(
            docs = ["SPL Token program to manage liquidity tokens. Either classic or 2022"],
            name = "liquidity_token_program",
        ),
        program(name = "superlendy_program", id = super_lendy::SUPER_LENDY_ID, docs = ["Superlendy Program"]),
        list(
            name = "reserves",
            flags(writable),
            checks(owner = SUPER_LENDY_ID),
            count_field = reserves_count,
        ),
        list(
            name = "reserve_liquidity_supplies",
            flags(writable),
            count_field = reserve_liquidity_supplies_count,
        ),
        list(
            name = "lp_token_mints",
            flags(writable),
            checks(owner = spl_token::ID),
            count_field = lp_token_mints_count,
        ),
        list(
            name = "lp_token_storages",
            flags(writable),
            checks(owner = spl_token::ID),
            count_field = lp_token_storages_count,
        )
    )]
    ClaimFees {
        #[instruction_builder(internal)]
        reserves_count: u8,
        #[instruction_builder(internal)]
        reserve_liquidity_supplies_count: u8,
        #[instruction_builder(internal)]
        lp_token_mints_count: u8,
        #[instruction_builder(internal)]
        lp_token_storages_count: u8,
        /// Minimal acceptable liquidity amount to claim as fee
        min_fee_amount: u64,
    },
}
