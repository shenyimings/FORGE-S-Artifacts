#![allow(unexpected_cfgs)]
use super::*;
///[VaultInstruction::Version] Builder struct
pub struct Version {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    pub print_version: bool,
}
impl Version {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self { #[cfg(feature = "program-id-manually")] program_id, print_version } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        #[allow(unused_mut)]
        let mut accounts = vec![];
        let ix = VaultInstruction::Version {
            print_version,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::CreateVault] Builder struct
pub struct CreateVault {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Vault account to create.
    pub vault: solana_program::pubkey::Pubkey,
    ///Authority assigned by Curator to manage Vaults.
    pub curator_vaults_authority: solana_program::pubkey::Pubkey,
    ///Curator of the Vault.
    pub curator: solana_program::pubkey::Pubkey,
    ///Liquidity tokens mint
    pub liquidity_mint: solana_program::pubkey::Pubkey,
    pub liquidity_buffer: solana_program::pubkey::Pubkey,
    ///SPL Token program to manage liquidity tokens. Either classic or 2022
    pub liquidity_token_program: solana_program::pubkey::Pubkey,
    pub reserves: Vec<solana_program::pubkey::Pubkey>,
    pub lp_token_mints: Vec<solana_program::pubkey::Pubkey>,
    pub lp_token_storages: Vec<solana_program::pubkey::Pubkey>,
    pub params: VaultParams,
}
impl CreateVault {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            vault,
            curator_vaults_authority,
            curator,
            liquidity_mint,
            liquidity_buffer,
            liquidity_token_program,
            reserves,
            lp_token_mints,
            lp_token_storages,
            params,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        #[allow(unused_mut)]
        let mut reserves_count;
        #[allow(unused_mut)]
        let mut lp_token_mints_count;
        #[allow(unused_mut)]
        let mut lp_token_storages_count;
        let (program_authority, _) = solana_program::pubkey::Pubkey::find_program_address(
            &[#[allow(clippy::useless_asref)] AUTHORITY_SEED.as_ref()],
            &program_id,
        );
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts.extend([solana_program::instruction::AccountMeta::new(vault, true)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(
                    curator_vaults_authority,
                    true,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(curator, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    program_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    liquidity_mint,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(liquidity_buffer, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    spl_token::ID,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    liquidity_token_program,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    solana_program::system_program::ID,
                    false,
                ),
            ]);
        accounts
            .extend({
                let keys = {
                    reserves_count = reserves
                        .len()
                        .try_into()
                        .expect(
                            concat!("convert ", stringify!(reserves), " accounts length"),
                        );
                    &reserves
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new_readonly(
                        *addr,
                        false,
                    ))
            });
        accounts
            .extend({
                let keys = {
                    lp_token_mints_count = lp_token_mints
                        .len()
                        .try_into()
                        .expect(
                            concat!(
                                "convert ", stringify!(lp_token_mints), " accounts length"
                            ),
                        );
                    &lp_token_mints
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new_readonly(
                        *addr,
                        false,
                    ))
            });
        accounts
            .extend({
                let keys = {
                    lp_token_storages_count = lp_token_storages
                        .len()
                        .try_into()
                        .expect(
                            concat!(
                                "convert ", stringify!(lp_token_storages),
                                " accounts length"
                            ),
                        );
                    &lp_token_storages
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        let ix = VaultInstruction::CreateVault {
            reserves_count,
            lp_token_mints_count,
            lp_token_storages_count,
            params,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::AlterVault] Builder struct
pub struct AlterVault {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Vault account to change.
    pub vault: solana_program::pubkey::Pubkey,
    ///Authority assigned by Curator to manage Vaults.
    pub curator_vaults_authority: solana_program::pubkey::Pubkey,
    ///Authority who can change percentage in distribution.
    pub distribution_manager: solana_program::pubkey::Pubkey,
    ///Curator of the Vault.
    pub curator: solana_program::pubkey::Pubkey,
    pub params: VaultParams,
}
impl AlterVault {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            vault,
            curator_vaults_authority,
            distribution_manager,
            curator,
            params,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts.extend([solana_program::instruction::AccountMeta::new(vault, false)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(
                    curator_vaults_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(
                    distribution_manager,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(curator, false),
            ]);
        let ix = VaultInstruction::AlterVault {
            params,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::ProposeDistributionItem] Builder struct
pub struct ProposeDistributionItem {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Vault account to change.
    pub vault: solana_program::pubkey::Pubkey,
    ///Authority assigned by Curator to manage Vaults.
    pub curator_vaults_authority: solana_program::pubkey::Pubkey,
    ///Curator of the Vault.
    pub curator: solana_program::pubkey::Pubkey,
    ///Reserve to be added in to distribution
    pub reserve: solana_program::pubkey::Pubkey,
    pub index: u8,
}
impl ProposeDistributionItem {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            vault,
            curator_vaults_authority,
            curator,
            reserve,
            index,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts.extend([solana_program::instruction::AccountMeta::new(vault, false)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(
                    curator_vaults_authority,
                    true,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(curator, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(reserve, false),
            ]);
        let ix = VaultInstruction::ProposeDistributionItem {
            index,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::AddDistributionItem] Builder struct
pub struct AddDistributionItem {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Vault account to change.
    pub vault: solana_program::pubkey::Pubkey,
    ///Authority assigned by Curator to manage Vaults.
    pub curator_vaults_authority: solana_program::pubkey::Pubkey,
    ///Curator of the Vault.
    pub curator: solana_program::pubkey::Pubkey,
    ///Mint of the reserve's LP tokens
    pub lp_token_mint: solana_program::pubkey::Pubkey,
    ///LP token storage to be used by Vault. Unitialized.
    pub lp_token_storage: solana_program::pubkey::Pubkey,
    pub index: u8,
}
impl AddDistributionItem {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            vault,
            curator_vaults_authority,
            curator,
            lp_token_mint,
            lp_token_storage,
            index,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        let (program_authority, _) = solana_program::pubkey::Pubkey::find_program_address(
            &[#[allow(clippy::useless_asref)] AUTHORITY_SEED.as_ref()],
            &program_id,
        );
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts.extend([solana_program::instruction::AccountMeta::new(vault, false)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(
                    curator_vaults_authority,
                    true,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(curator, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    lp_token_mint,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(lp_token_storage, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    program_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    spl_token::ID,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    solana_program::system_program::ID,
                    false,
                ),
            ]);
        let ix = VaultInstruction::AddDistributionItem {
            index,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::RemoveDistributionItem] Builder struct
pub struct RemoveDistributionItem {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Vault account to change.
    pub vault: solana_program::pubkey::Pubkey,
    ///Authority assigned by Curator to manage Vaults.
    pub curator_vaults_authority: solana_program::pubkey::Pubkey,
    ///Curator of the Vault.
    pub curator: solana_program::pubkey::Pubkey,
    ///LP token storage.
    pub lp_token_storage: solana_program::pubkey::Pubkey,
    pub reserve: Pubkey,
}
impl RemoveDistributionItem {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            vault,
            curator_vaults_authority,
            curator,
            lp_token_storage,
            reserve,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        let (program_authority, _) = solana_program::pubkey::Pubkey::find_program_address(
            &[#[allow(clippy::useless_asref)] AUTHORITY_SEED.as_ref()],
            &program_id,
        );
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts.extend([solana_program::instruction::AccountMeta::new(vault, false)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(
                    curator_vaults_authority,
                    true,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(curator, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(lp_token_storage, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    program_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    spl_token::ID,
                    false,
                ),
            ]);
        let ix = VaultInstruction::RemoveDistributionItem {
            reserve,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::CreatePosition] Builder struct
pub struct CreatePosition {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Position account to create.
    pub position: solana_program::pubkey::Pubkey,
    ///Vault account the position belongs to.
    pub vault: solana_program::pubkey::Pubkey,
    ///Position owner.
    pub owner: solana_program::pubkey::Pubkey,
}
impl CreatePosition {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            position,
            vault,
            owner,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        let (program_authority, _) = solana_program::pubkey::Pubkey::find_program_address(
            &[#[allow(clippy::useless_asref)] AUTHORITY_SEED.as_ref()],
            &program_id,
        );
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts.extend([solana_program::instruction::AccountMeta::new(position, true)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(vault, false),
            ]);
        accounts.extend([solana_program::instruction::AccountMeta::new(owner, true)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    program_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    solana_program::system_program::ID,
                    false,
                ),
            ]);
        let ix = VaultInstruction::CreatePosition {
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::Deposit] Builder struct
pub struct Deposit {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Vault account to deposit to.
    pub vault: solana_program::pubkey::Pubkey,
    ///Position account to deposit thru.
    pub position: solana_program::pubkey::Pubkey,
    ///Position owner.
    pub owner: solana_program::pubkey::Pubkey,
    ///Liquidity tokens mint
    pub liquidity_mint: solana_program::pubkey::Pubkey,
    pub liquidity_source: solana_program::pubkey::Pubkey,
    pub liquidity_buffer: solana_program::pubkey::Pubkey,
    pub superlendy_program_authority: solana_program::pubkey::Pubkey,
    ///SPL Token program to manage liquidity tokens. Either classic or 2022
    pub liquidity_token_program: solana_program::pubkey::Pubkey,
    pub reserves: Vec<solana_program::pubkey::Pubkey>,
    pub reserve_liquidity_supplies: Vec<solana_program::pubkey::Pubkey>,
    pub lp_token_mints: Vec<solana_program::pubkey::Pubkey>,
    pub lp_token_storages: Vec<solana_program::pubkey::Pubkey>,
    pub amount: u64,
}
impl Deposit {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            vault,
            position,
            owner,
            liquidity_mint,
            liquidity_source,
            liquidity_buffer,
            superlendy_program_authority,
            liquidity_token_program,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
            amount,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        #[allow(unused_mut)]
        let mut reserves_count;
        #[allow(unused_mut)]
        let mut reserve_liquidity_supplies_count;
        #[allow(unused_mut)]
        let mut lp_token_mints_count;
        #[allow(unused_mut)]
        let mut lp_token_storages_count;
        let (program_authority, _) = solana_program::pubkey::Pubkey::find_program_address(
            &[#[allow(clippy::useless_asref)] AUTHORITY_SEED.as_ref()],
            &program_id,
        );
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts.extend([solana_program::instruction::AccountMeta::new(vault, false)]);
        accounts
            .extend([solana_program::instruction::AccountMeta::new(position, false)]);
        accounts.extend([solana_program::instruction::AccountMeta::new(owner, true)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    liquidity_mint,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(liquidity_source, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(liquidity_buffer, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    program_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    superlendy_program_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    spl_token::ID,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    liquidity_token_program,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    super_lendy::SUPER_LENDY_ID,
                    false,
                ),
            ]);
        accounts
            .extend({
                let keys = {
                    reserves_count = reserves
                        .len()
                        .try_into()
                        .expect(
                            concat!("convert ", stringify!(reserves), " accounts length"),
                        );
                    &reserves
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        accounts
            .extend({
                let keys = {
                    reserve_liquidity_supplies_count = reserve_liquidity_supplies
                        .len()
                        .try_into()
                        .expect(
                            concat!(
                                "convert ", stringify!(reserve_liquidity_supplies),
                                " accounts length"
                            ),
                        );
                    &reserve_liquidity_supplies
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        accounts
            .extend({
                let keys = {
                    lp_token_mints_count = lp_token_mints
                        .len()
                        .try_into()
                        .expect(
                            concat!(
                                "convert ", stringify!(lp_token_mints), " accounts length"
                            ),
                        );
                    &lp_token_mints
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        accounts
            .extend({
                let keys = {
                    lp_token_storages_count = lp_token_storages
                        .len()
                        .try_into()
                        .expect(
                            concat!(
                                "convert ", stringify!(lp_token_storages),
                                " accounts length"
                            ),
                        );
                    &lp_token_storages
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        let ix = VaultInstruction::Deposit {
            reserves_count,
            reserve_liquidity_supplies_count,
            lp_token_mints_count,
            lp_token_storages_count,
            amount,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::Withdraw] Builder struct
pub struct Withdraw {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Vault account to withdraw from.
    pub vault: solana_program::pubkey::Pubkey,
    ///Position account to withdraw from.
    pub position: solana_program::pubkey::Pubkey,
    ///Position owner.
    pub owner: solana_program::pubkey::Pubkey,
    ///Liquidity tokens mint
    pub liquidity_mint: solana_program::pubkey::Pubkey,
    pub liquidity_destination: solana_program::pubkey::Pubkey,
    pub liquidity_buffer: solana_program::pubkey::Pubkey,
    pub superlendy_program_authority: solana_program::pubkey::Pubkey,
    ///SPL Token program to manage liquidity tokens. Either classic or 2022
    pub liquidity_token_program: solana_program::pubkey::Pubkey,
    pub reserves: Vec<solana_program::pubkey::Pubkey>,
    pub reserve_liquidity_supplies: Vec<solana_program::pubkey::Pubkey>,
    pub lp_token_mints: Vec<solana_program::pubkey::Pubkey>,
    pub lp_token_storages: Vec<solana_program::pubkey::Pubkey>,
    /// Desired VLP amount to redeem
    pub vlp_amount: u64,
}
impl Withdraw {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            vault,
            position,
            owner,
            liquidity_mint,
            liquidity_destination,
            liquidity_buffer,
            superlendy_program_authority,
            liquidity_token_program,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
            vlp_amount,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        #[allow(unused_mut)]
        let mut reserves_count;
        #[allow(unused_mut)]
        let mut reserve_liquidity_supplies_count;
        #[allow(unused_mut)]
        let mut lp_token_mints_count;
        #[allow(unused_mut)]
        let mut lp_token_storages_count;
        let (program_authority, _) = solana_program::pubkey::Pubkey::find_program_address(
            &[#[allow(clippy::useless_asref)] AUTHORITY_SEED.as_ref()],
            &program_id,
        );
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts.extend([solana_program::instruction::AccountMeta::new(vault, false)]);
        accounts
            .extend([solana_program::instruction::AccountMeta::new(position, false)]);
        accounts.extend([solana_program::instruction::AccountMeta::new(owner, true)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    liquidity_mint,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(
                    liquidity_destination,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(liquidity_buffer, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    program_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    superlendy_program_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    spl_token::ID,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    liquidity_token_program,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    super_lendy::SUPER_LENDY_ID,
                    false,
                ),
            ]);
        accounts
            .extend({
                let keys = {
                    reserves_count = reserves
                        .len()
                        .try_into()
                        .expect(
                            concat!("convert ", stringify!(reserves), " accounts length"),
                        );
                    &reserves
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        accounts
            .extend({
                let keys = {
                    reserve_liquidity_supplies_count = reserve_liquidity_supplies
                        .len()
                        .try_into()
                        .expect(
                            concat!(
                                "convert ", stringify!(reserve_liquidity_supplies),
                                " accounts length"
                            ),
                        );
                    &reserve_liquidity_supplies
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        accounts
            .extend({
                let keys = {
                    lp_token_mints_count = lp_token_mints
                        .len()
                        .try_into()
                        .expect(
                            concat!(
                                "convert ", stringify!(lp_token_mints), " accounts length"
                            ),
                        );
                    &lp_token_mints
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        accounts
            .extend({
                let keys = {
                    lp_token_storages_count = lp_token_storages
                        .len()
                        .try_into()
                        .expect(
                            concat!(
                                "convert ", stringify!(lp_token_storages),
                                " accounts length"
                            ),
                        );
                    &lp_token_storages
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        let ix = VaultInstruction::Withdraw {
            reserves_count,
            reserve_liquidity_supplies_count,
            lp_token_mints_count,
            lp_token_storages_count,
            vlp_amount,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::ClosePosition] Builder struct
pub struct ClosePosition {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Position account to close.
    pub position: solana_program::pubkey::Pubkey,
    pub vault: solana_program::pubkey::Pubkey,
    ///Owner of the position
    pub owner: solana_program::pubkey::Pubkey,
}
impl ClosePosition {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            position,
            vault,
            owner,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts
            .extend([solana_program::instruction::AccountMeta::new(position, false)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(vault, false),
            ]);
        accounts.extend([solana_program::instruction::AccountMeta::new(owner, true)]);
        let ix = VaultInstruction::ClosePosition {};
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::Rebalance] Builder struct
pub struct Rebalance {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    pub vault: solana_program::pubkey::Pubkey,
    ///Liquidity tokens mint
    pub liquidity_mint: solana_program::pubkey::Pubkey,
    pub liquidity_buffer: solana_program::pubkey::Pubkey,
    pub superlendy_program_authority: solana_program::pubkey::Pubkey,
    ///SPL Token program to manage liquidity tokens. Either classic or 2022
    pub liquidity_token_program: solana_program::pubkey::Pubkey,
    pub reserves: Vec<solana_program::pubkey::Pubkey>,
    pub reserve_liquidity_supplies: Vec<solana_program::pubkey::Pubkey>,
    pub lp_token_mints: Vec<solana_program::pubkey::Pubkey>,
    pub lp_token_storages: Vec<solana_program::pubkey::Pubkey>,
}
impl Rebalance {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            vault,
            liquidity_mint,
            liquidity_buffer,
            superlendy_program_authority,
            liquidity_token_program,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        #[allow(unused_mut)]
        let mut reserves_count;
        #[allow(unused_mut)]
        let mut reserve_liquidity_supplies_count;
        #[allow(unused_mut)]
        let mut lp_token_mints_count;
        #[allow(unused_mut)]
        let mut lp_token_storages_count;
        let (program_authority, _) = solana_program::pubkey::Pubkey::find_program_address(
            &[#[allow(clippy::useless_asref)] AUTHORITY_SEED.as_ref()],
            &program_id,
        );
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(vault, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    program_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    liquidity_mint,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(liquidity_buffer, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    superlendy_program_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    spl_token::ID,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    liquidity_token_program,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    super_lendy::SUPER_LENDY_ID,
                    false,
                ),
            ]);
        accounts
            .extend({
                let keys = {
                    reserves_count = reserves
                        .len()
                        .try_into()
                        .expect(
                            concat!("convert ", stringify!(reserves), " accounts length"),
                        );
                    &reserves
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        accounts
            .extend({
                let keys = {
                    reserve_liquidity_supplies_count = reserve_liquidity_supplies
                        .len()
                        .try_into()
                        .expect(
                            concat!(
                                "convert ", stringify!(reserve_liquidity_supplies),
                                " accounts length"
                            ),
                        );
                    &reserve_liquidity_supplies
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        accounts
            .extend({
                let keys = {
                    lp_token_mints_count = lp_token_mints
                        .len()
                        .try_into()
                        .expect(
                            concat!(
                                "convert ", stringify!(lp_token_mints), " accounts length"
                            ),
                        );
                    &lp_token_mints
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        accounts
            .extend({
                let keys = {
                    lp_token_storages_count = lp_token_storages
                        .len()
                        .try_into()
                        .expect(
                            concat!(
                                "convert ", stringify!(lp_token_storages),
                                " accounts length"
                            ),
                        );
                    &lp_token_storages
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        let ix = VaultInstruction::Rebalance {
            reserves_count,
            reserve_liquidity_supplies_count,
            lp_token_mints_count,
            lp_token_storages_count,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::InitRewardSupply] Builder struct
pub struct InitRewardSupply {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Reward token mint.
    pub reward_mint: solana_program::pubkey::Pubkey,
    ///Vault to init reward supply for.
    pub vault: solana_program::pubkey::Pubkey,
    ///Authority who can manage Vault.
    pub curator_vaults_authority: solana_program::pubkey::Pubkey,
    ///Curator account.
    pub curator: solana_program::pubkey::Pubkey,
    ///SPL Token program
    pub token_program: solana_program::pubkey::Pubkey,
}
impl InitRewardSupply {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            reward_mint,
            vault,
            curator_vaults_authority,
            curator,
            token_program,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        let (reward_supply, _) = solana_program::pubkey::Pubkey::find_program_address(
            &[
                #[allow(clippy::useless_asref)]
                vault.as_ref(),
                #[allow(clippy::useless_asref)]
                reward_mint.as_ref(),
                #[allow(clippy::useless_asref)]
                crate::pda::REWARD_SUPPLY_SEED.as_ref(),
            ],
            &program_id,
        );
        let (reward_authority, _) = solana_program::pubkey::Pubkey::find_program_address(
            &[
                #[allow(clippy::useless_asref)]
                vault.as_ref(),
                #[allow(clippy::useless_asref)]
                crate::pda::AUTHORITY_SEED.as_ref(),
            ],
            &program_id,
        );
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(reward_supply, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    reward_mint,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(vault, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(
                    curator_vaults_authority,
                    true,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(curator, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    reward_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    token_program,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    solana_program::system_program::ID,
                    false,
                ),
            ]);
        let ix = VaultInstruction::InitRewardSupply {
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::SetRewardRules] Builder struct
pub struct SetRewardRules {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Vault account to set reward rules for
    pub vault: solana_program::pubkey::Pubkey,
    ///Authority who can configure Vault.
    pub curator_vaults_authority: solana_program::pubkey::Pubkey,
    ///Curator account.
    pub curator: solana_program::pubkey::Pubkey,
    pub rules: RewardRules,
}
impl SetRewardRules {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            vault,
            curator_vaults_authority,
            curator,
            rules,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts.extend([solana_program::instruction::AccountMeta::new(vault, false)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(
                    curator_vaults_authority,
                    true,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(curator, false),
            ]);
        let ix = VaultInstruction::SetRewardRules {
            rules,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::ClaimReward] Builder struct
pub struct ClaimReward {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Position account to claim rewords for. Refreshed.
    pub position: solana_program::pubkey::Pubkey,
    ///User's SPL token wallet which will receive reward tokens
    pub destination_wallet: solana_program::pubkey::Pubkey,
    ///Position owner
    pub position_owner: solana_program::pubkey::Pubkey,
    ///Vault, position belongs to.
    pub vault: solana_program::pubkey::Pubkey,
    ///Reward token mint to claim. Determines which reward will be claimed from the Position.
    pub reward_mint: solana_program::pubkey::Pubkey,
    ///SPL Token program
    pub token_program: solana_program::pubkey::Pubkey,
}
impl ClaimReward {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            position,
            destination_wallet,
            position_owner,
            vault,
            reward_mint,
            token_program,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        let (rewards_supply, _) = solana_program::pubkey::Pubkey::find_program_address(
            &[
                #[allow(clippy::useless_asref)]
                vault.as_ref(),
                #[allow(clippy::useless_asref)]
                reward_mint.as_ref(),
                #[allow(clippy::useless_asref)]
                crate::pda::REWARD_SUPPLY_SEED.as_ref(),
            ],
            &program_id,
        );
        let (reward_authority, _) = solana_program::pubkey::Pubkey::find_program_address(
            &[
                #[allow(clippy::useless_asref)]
                vault.as_ref(),
                #[allow(clippy::useless_asref)]
                crate::pda::AUTHORITY_SEED.as_ref(),
            ],
            &program_id,
        );
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts
            .extend([solana_program::instruction::AccountMeta::new(position, false)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(rewards_supply, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(destination_wallet, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    position_owner,
                    true,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(vault, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    reward_mint,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    reward_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    token_program,
                    false,
                ),
            ]);
        let ix = VaultInstruction::ClaimReward {};
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::WithdrawReward] Builder struct
pub struct WithdrawReward {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///SPL token account of reward_mint to receive reward tokens.
    pub destination_wallet: solana_program::pubkey::Pubkey,
    ///Vault to withdraw rewards from.
    pub vault: solana_program::pubkey::Pubkey,
    ///Authority who can configure Vault.
    pub curator_vaults_authority: solana_program::pubkey::Pubkey,
    ///Curator account.
    pub curator: solana_program::pubkey::Pubkey,
    ///Reward token mint to withdraw.
    pub reward_mint: solana_program::pubkey::Pubkey,
    ///SPL Token program
    pub token_program: solana_program::pubkey::Pubkey,
    pub amount: u64,
}
impl WithdrawReward {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            destination_wallet,
            vault,
            curator_vaults_authority,
            curator,
            reward_mint,
            token_program,
            amount,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        let (rewards_supply, _) = solana_program::pubkey::Pubkey::find_program_address(
            &[
                #[allow(clippy::useless_asref)]
                vault.as_ref(),
                #[allow(clippy::useless_asref)]
                reward_mint.as_ref(),
                #[allow(clippy::useless_asref)]
                crate::pda::REWARD_SUPPLY_SEED.as_ref(),
            ],
            &program_id,
        );
        let (reward_authority, _) = solana_program::pubkey::Pubkey::find_program_address(
            &[
                #[allow(clippy::useless_asref)]
                vault.as_ref(),
                #[allow(clippy::useless_asref)]
                crate::pda::AUTHORITY_SEED.as_ref(),
            ],
            &program_id,
        );
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(rewards_supply, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(destination_wallet, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(vault, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(
                    curator_vaults_authority,
                    true,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(curator, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    reward_mint,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    reward_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    token_program,
                    false,
                ),
            ]);
        let ix = VaultInstruction::WithdrawReward {
            amount,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[VaultInstruction::ClaimFees] Builder struct
pub struct ClaimFees {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Vault account to claim fees from.
    pub vault: solana_program::pubkey::Pubkey,
    ///Authority assigned by Curator to manage Vaults.
    pub curator_vaults_authority: solana_program::pubkey::Pubkey,
    ///Curator of the Vault.
    pub curator: solana_program::pubkey::Pubkey,
    ///Liquidity tokens mint
    pub liquidity_mint: solana_program::pubkey::Pubkey,
    ///Any SPL account of proper mint to transfer fees to
    pub fee_receiver: solana_program::pubkey::Pubkey,
    ///Any SPL account of proper mint to transfer fees to
    pub liquidity_buffer: solana_program::pubkey::Pubkey,
    pub superlendy_program_authority: solana_program::pubkey::Pubkey,
    ///SPL Token program to manage liquidity tokens. Either classic or 2022
    pub liquidity_token_program: solana_program::pubkey::Pubkey,
    pub reserves: Vec<solana_program::pubkey::Pubkey>,
    pub reserve_liquidity_supplies: Vec<solana_program::pubkey::Pubkey>,
    pub lp_token_mints: Vec<solana_program::pubkey::Pubkey>,
    pub lp_token_storages: Vec<solana_program::pubkey::Pubkey>,
    /// Minimal acceptable liquidity amount to claim as fee
    pub min_fee_amount: u64,
}
impl ClaimFees {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            vault,
            curator_vaults_authority,
            curator,
            liquidity_mint,
            fee_receiver,
            liquidity_buffer,
            superlendy_program_authority,
            liquidity_token_program,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
            min_fee_amount,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = VAULT_ID;
        #[allow(unused_mut)]
        let mut reserves_count;
        #[allow(unused_mut)]
        let mut reserve_liquidity_supplies_count;
        #[allow(unused_mut)]
        let mut lp_token_mints_count;
        #[allow(unused_mut)]
        let mut lp_token_storages_count;
        let (program_authority, _) = solana_program::pubkey::Pubkey::find_program_address(
            &[#[allow(clippy::useless_asref)] AUTHORITY_SEED.as_ref()],
            &program_id,
        );
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts.extend([solana_program::instruction::AccountMeta::new(vault, false)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    curator_vaults_authority,
                    true,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(curator, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    liquidity_mint,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(fee_receiver, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(liquidity_buffer, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    program_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    superlendy_program_authority,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    spl_token::ID,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    liquidity_token_program,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    super_lendy::SUPER_LENDY_ID,
                    false,
                ),
            ]);
        accounts
            .extend({
                let keys = {
                    reserves_count = reserves
                        .len()
                        .try_into()
                        .expect(
                            concat!("convert ", stringify!(reserves), " accounts length"),
                        );
                    &reserves
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        accounts
            .extend({
                let keys = {
                    reserve_liquidity_supplies_count = reserve_liquidity_supplies
                        .len()
                        .try_into()
                        .expect(
                            concat!(
                                "convert ", stringify!(reserve_liquidity_supplies),
                                " accounts length"
                            ),
                        );
                    &reserve_liquidity_supplies
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        accounts
            .extend({
                let keys = {
                    lp_token_mints_count = lp_token_mints
                        .len()
                        .try_into()
                        .expect(
                            concat!(
                                "convert ", stringify!(lp_token_mints), " accounts length"
                            ),
                        );
                    &lp_token_mints
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        accounts
            .extend({
                let keys = {
                    lp_token_storages_count = lp_token_storages
                        .len()
                        .try_into()
                        .expect(
                            concat!(
                                "convert ", stringify!(lp_token_storages),
                                " accounts length"
                            ),
                        );
                    &lp_token_storages
                };
                #[allow(clippy::into_iter_on_ref)]
                keys.into_iter()
                    .map(|addr| solana_program::instruction::AccountMeta::new(
                        *addr,
                        false,
                    ))
            });
        let ix = VaultInstruction::ClaimFees {
            reserves_count,
            reserve_liquidity_supplies_count,
            lp_token_mints_count,
            lp_token_storages_count,
            min_fee_amount,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
/// [VaultInstruction::Version] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct VersionAccountIndexes {}
impl VersionAccountIndexes {
    pub const COUNT: usize = 0usize;
    pub fn new_direct_order() -> Self {
        Self {}
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let _ = indexes;
        Ok(Self {})
    }
}
impl<'a> TryFrom<&'a [u8]> for VersionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for VersionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for VersionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for VersionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [VaultInstruction::CreateVault] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct CreateVaultAccountIndexes {
    pub vault: usize,
    pub curator_vaults_authority: usize,
    pub curator: usize,
    pub program_authority: usize,
    pub liquidity_mint: usize,
    pub liquidity_buffer: usize,
    pub lp_token_program: usize,
    pub liquidity_token_program: usize,
    pub system_program: usize,
    pub reserves: Vec<usize>,
    pub lp_token_mints: Vec<usize>,
    pub lp_token_storages: Vec<usize>,
}
impl CreateVaultAccountIndexes {
    pub const VAULT: usize = 0usize;
    pub const CURATOR_VAULTS_AUTHORITY: usize = 1usize;
    pub const CURATOR: usize = 2usize;
    pub const PROGRAM_AUTHORITY: usize = 3usize;
    pub const LIQUIDITY_MINT: usize = 4usize;
    pub const LIQUIDITY_BUFFER: usize = 5usize;
    pub const LP_TOKEN_PROGRAM: usize = 6usize;
    pub const LIQUIDITY_TOKEN_PROGRAM: usize = 7usize;
    pub const SYSTEM_PROGRAM: usize = 8usize;
    pub fn new_direct_order(
        reserves_size: usize,
        lp_token_mints_size: usize,
        lp_token_storages_size: usize,
    ) -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            vault: iter.next().unwrap(),
            curator_vaults_authority: iter.next().unwrap(),
            curator: iter.next().unwrap(),
            program_authority: iter.next().unwrap(),
            liquidity_mint: iter.next().unwrap(),
            liquidity_buffer: iter.next().unwrap(),
            lp_token_program: iter.next().unwrap(),
            liquidity_token_program: iter.next().unwrap(),
            system_program: iter.next().unwrap(),
            reserves: {
                let mut out = vec![];
                for _ in 0..reserves_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
            lp_token_mints: {
                let mut out = vec![];
                for _ in 0..lp_token_mints_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
            lp_token_storages: {
                let mut out = vec![];
                for _ in 0..lp_token_storages_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
        reserves_size: usize,
        lp_token_mints_size: usize,
        lp_token_storages_size: usize,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator_vaults_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            program_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_mint: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_buffer: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            lp_token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            system_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            reserves: {
                let mut out = vec![];
                for _ in 0..reserves_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
            lp_token_mints: {
                let mut out = vec![];
                for _ in 0..lp_token_mints_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
            lp_token_storages: {
                let mut out = vec![];
                for _ in 0..lp_token_storages_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
        })
    }
}
/// [VaultInstruction::AlterVault] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct AlterVaultAccountIndexes {
    pub vault: usize,
    pub curator_vaults_authority: usize,
    pub distribution_manager: usize,
    pub curator: usize,
}
impl AlterVaultAccountIndexes {
    pub const COUNT: usize = 4usize;
    pub const VAULT: usize = 0usize;
    pub const CURATOR_VAULTS_AUTHORITY: usize = 1usize;
    pub const DISTRIBUTION_MANAGER: usize = 2usize;
    pub const CURATOR: usize = 3usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            vault: iter.next().unwrap(),
            curator_vaults_authority: iter.next().unwrap(),
            distribution_manager: iter.next().unwrap(),
            curator: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator_vaults_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            distribution_manager: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for AlterVaultAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for AlterVaultAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for AlterVaultAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for AlterVaultAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [VaultInstruction::ProposeDistributionItem] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct ProposeDistributionItemAccountIndexes {
    pub vault: usize,
    pub curator_vaults_authority: usize,
    pub curator: usize,
    pub reserve: usize,
}
impl ProposeDistributionItemAccountIndexes {
    pub const COUNT: usize = 4usize;
    pub const VAULT: usize = 0usize;
    pub const CURATOR_VAULTS_AUTHORITY: usize = 1usize;
    pub const CURATOR: usize = 2usize;
    pub const RESERVE: usize = 3usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            vault: iter.next().unwrap(),
            curator_vaults_authority: iter.next().unwrap(),
            curator: iter.next().unwrap(),
            reserve: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator_vaults_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            reserve: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for ProposeDistributionItemAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for ProposeDistributionItemAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for ProposeDistributionItemAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for ProposeDistributionItemAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [VaultInstruction::AddDistributionItem] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct AddDistributionItemAccountIndexes {
    pub vault: usize,
    pub curator_vaults_authority: usize,
    pub curator: usize,
    pub lp_token_mint: usize,
    pub lp_token_storage: usize,
    pub program_authority: usize,
    pub token_program: usize,
    pub system_program: usize,
}
impl AddDistributionItemAccountIndexes {
    pub const COUNT: usize = 8usize;
    pub const VAULT: usize = 0usize;
    pub const CURATOR_VAULTS_AUTHORITY: usize = 1usize;
    pub const CURATOR: usize = 2usize;
    pub const LP_TOKEN_MINT: usize = 3usize;
    pub const LP_TOKEN_STORAGE: usize = 4usize;
    pub const PROGRAM_AUTHORITY: usize = 5usize;
    pub const TOKEN_PROGRAM: usize = 6usize;
    pub const SYSTEM_PROGRAM: usize = 7usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            vault: iter.next().unwrap(),
            curator_vaults_authority: iter.next().unwrap(),
            curator: iter.next().unwrap(),
            lp_token_mint: iter.next().unwrap(),
            lp_token_storage: iter.next().unwrap(),
            program_authority: iter.next().unwrap(),
            token_program: iter.next().unwrap(),
            system_program: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator_vaults_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            lp_token_mint: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            lp_token_storage: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            program_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            system_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for AddDistributionItemAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for AddDistributionItemAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for AddDistributionItemAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for AddDistributionItemAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [VaultInstruction::RemoveDistributionItem] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct RemoveDistributionItemAccountIndexes {
    pub vault: usize,
    pub curator_vaults_authority: usize,
    pub curator: usize,
    pub lp_token_storage: usize,
    pub program_authority: usize,
    pub token_program: usize,
}
impl RemoveDistributionItemAccountIndexes {
    pub const COUNT: usize = 6usize;
    pub const VAULT: usize = 0usize;
    pub const CURATOR_VAULTS_AUTHORITY: usize = 1usize;
    pub const CURATOR: usize = 2usize;
    pub const LP_TOKEN_STORAGE: usize = 3usize;
    pub const PROGRAM_AUTHORITY: usize = 4usize;
    pub const TOKEN_PROGRAM: usize = 5usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            vault: iter.next().unwrap(),
            curator_vaults_authority: iter.next().unwrap(),
            curator: iter.next().unwrap(),
            lp_token_storage: iter.next().unwrap(),
            program_authority: iter.next().unwrap(),
            token_program: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator_vaults_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            lp_token_storage: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            program_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for RemoveDistributionItemAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for RemoveDistributionItemAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for RemoveDistributionItemAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for RemoveDistributionItemAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [VaultInstruction::CreatePosition] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct CreatePositionAccountIndexes {
    pub position: usize,
    pub vault: usize,
    pub owner: usize,
    pub program_authority: usize,
    pub system_program: usize,
}
impl CreatePositionAccountIndexes {
    pub const COUNT: usize = 5usize;
    pub const POSITION: usize = 0usize;
    pub const VAULT: usize = 1usize;
    pub const OWNER: usize = 2usize;
    pub const PROGRAM_AUTHORITY: usize = 3usize;
    pub const SYSTEM_PROGRAM: usize = 4usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            position: iter.next().unwrap(),
            vault: iter.next().unwrap(),
            owner: iter.next().unwrap(),
            program_authority: iter.next().unwrap(),
            system_program: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            position: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            owner: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            program_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            system_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for CreatePositionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for CreatePositionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for CreatePositionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for CreatePositionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [VaultInstruction::Deposit] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct DepositAccountIndexes {
    pub vault: usize,
    pub position: usize,
    pub owner: usize,
    pub liquidity_mint: usize,
    pub liquidity_source: usize,
    pub liquidity_buffer: usize,
    pub program_authority: usize,
    pub superlendy_program_authority: usize,
    pub lp_token_program: usize,
    pub liquidity_token_program: usize,
    pub superlendy_program: usize,
    pub reserves: Vec<usize>,
    pub reserve_liquidity_supplies: Vec<usize>,
    pub lp_token_mints: Vec<usize>,
    pub lp_token_storages: Vec<usize>,
}
impl DepositAccountIndexes {
    pub const VAULT: usize = 0usize;
    pub const POSITION: usize = 1usize;
    pub const OWNER: usize = 2usize;
    pub const LIQUIDITY_MINT: usize = 3usize;
    pub const LIQUIDITY_SOURCE: usize = 4usize;
    pub const LIQUIDITY_BUFFER: usize = 5usize;
    pub const PROGRAM_AUTHORITY: usize = 6usize;
    pub const SUPERLENDY_PROGRAM_AUTHORITY: usize = 7usize;
    pub const LP_TOKEN_PROGRAM: usize = 8usize;
    pub const LIQUIDITY_TOKEN_PROGRAM: usize = 9usize;
    pub const SUPERLENDY_PROGRAM: usize = 10usize;
    pub fn new_direct_order(
        reserves_size: usize,
        reserve_liquidity_supplies_size: usize,
        lp_token_mints_size: usize,
        lp_token_storages_size: usize,
    ) -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            vault: iter.next().unwrap(),
            position: iter.next().unwrap(),
            owner: iter.next().unwrap(),
            liquidity_mint: iter.next().unwrap(),
            liquidity_source: iter.next().unwrap(),
            liquidity_buffer: iter.next().unwrap(),
            program_authority: iter.next().unwrap(),
            superlendy_program_authority: iter.next().unwrap(),
            lp_token_program: iter.next().unwrap(),
            liquidity_token_program: iter.next().unwrap(),
            superlendy_program: iter.next().unwrap(),
            reserves: {
                let mut out = vec![];
                for _ in 0..reserves_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
            reserve_liquidity_supplies: {
                let mut out = vec![];
                for _ in 0..reserve_liquidity_supplies_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
            lp_token_mints: {
                let mut out = vec![];
                for _ in 0..lp_token_mints_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
            lp_token_storages: {
                let mut out = vec![];
                for _ in 0..lp_token_storages_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
        reserves_size: usize,
        reserve_liquidity_supplies_size: usize,
        lp_token_mints_size: usize,
        lp_token_storages_size: usize,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            position: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            owner: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_mint: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_source: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_buffer: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            program_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            superlendy_program_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            lp_token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            superlendy_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            reserves: {
                let mut out = vec![];
                for _ in 0..reserves_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
            reserve_liquidity_supplies: {
                let mut out = vec![];
                for _ in 0..reserve_liquidity_supplies_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
            lp_token_mints: {
                let mut out = vec![];
                for _ in 0..lp_token_mints_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
            lp_token_storages: {
                let mut out = vec![];
                for _ in 0..lp_token_storages_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
        })
    }
}
/// [VaultInstruction::Withdraw] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct WithdrawAccountIndexes {
    pub vault: usize,
    pub position: usize,
    pub owner: usize,
    pub liquidity_mint: usize,
    pub liquidity_destination: usize,
    pub liquidity_buffer: usize,
    pub program_authority: usize,
    pub superlendy_program_authority: usize,
    pub lp_token_program: usize,
    pub liquidity_token_program: usize,
    pub superlendy_program: usize,
    pub reserves: Vec<usize>,
    pub reserve_liquidity_supplies: Vec<usize>,
    pub lp_token_mints: Vec<usize>,
    pub lp_token_storages: Vec<usize>,
}
impl WithdrawAccountIndexes {
    pub const VAULT: usize = 0usize;
    pub const POSITION: usize = 1usize;
    pub const OWNER: usize = 2usize;
    pub const LIQUIDITY_MINT: usize = 3usize;
    pub const LIQUIDITY_DESTINATION: usize = 4usize;
    pub const LIQUIDITY_BUFFER: usize = 5usize;
    pub const PROGRAM_AUTHORITY: usize = 6usize;
    pub const SUPERLENDY_PROGRAM_AUTHORITY: usize = 7usize;
    pub const LP_TOKEN_PROGRAM: usize = 8usize;
    pub const LIQUIDITY_TOKEN_PROGRAM: usize = 9usize;
    pub const SUPERLENDY_PROGRAM: usize = 10usize;
    pub fn new_direct_order(
        reserves_size: usize,
        reserve_liquidity_supplies_size: usize,
        lp_token_mints_size: usize,
        lp_token_storages_size: usize,
    ) -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            vault: iter.next().unwrap(),
            position: iter.next().unwrap(),
            owner: iter.next().unwrap(),
            liquidity_mint: iter.next().unwrap(),
            liquidity_destination: iter.next().unwrap(),
            liquidity_buffer: iter.next().unwrap(),
            program_authority: iter.next().unwrap(),
            superlendy_program_authority: iter.next().unwrap(),
            lp_token_program: iter.next().unwrap(),
            liquidity_token_program: iter.next().unwrap(),
            superlendy_program: iter.next().unwrap(),
            reserves: {
                let mut out = vec![];
                for _ in 0..reserves_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
            reserve_liquidity_supplies: {
                let mut out = vec![];
                for _ in 0..reserve_liquidity_supplies_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
            lp_token_mints: {
                let mut out = vec![];
                for _ in 0..lp_token_mints_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
            lp_token_storages: {
                let mut out = vec![];
                for _ in 0..lp_token_storages_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
        reserves_size: usize,
        reserve_liquidity_supplies_size: usize,
        lp_token_mints_size: usize,
        lp_token_storages_size: usize,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            position: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            owner: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_mint: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_destination: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_buffer: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            program_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            superlendy_program_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            lp_token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            superlendy_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            reserves: {
                let mut out = vec![];
                for _ in 0..reserves_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
            reserve_liquidity_supplies: {
                let mut out = vec![];
                for _ in 0..reserve_liquidity_supplies_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
            lp_token_mints: {
                let mut out = vec![];
                for _ in 0..lp_token_mints_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
            lp_token_storages: {
                let mut out = vec![];
                for _ in 0..lp_token_storages_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
        })
    }
}
/// [VaultInstruction::ClosePosition] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct ClosePositionAccountIndexes {
    pub position: usize,
    pub vault: usize,
    pub owner: usize,
}
impl ClosePositionAccountIndexes {
    pub const COUNT: usize = 3usize;
    pub const POSITION: usize = 0usize;
    pub const VAULT: usize = 1usize;
    pub const OWNER: usize = 2usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            position: iter.next().unwrap(),
            vault: iter.next().unwrap(),
            owner: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            position: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            owner: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for ClosePositionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for ClosePositionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for ClosePositionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for ClosePositionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [VaultInstruction::Rebalance] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct RebalanceAccountIndexes {
    pub vault: usize,
    pub program_authority: usize,
    pub liquidity_mint: usize,
    pub liquidity_buffer: usize,
    pub superlendy_program_authority: usize,
    pub lp_token_program: usize,
    pub liquidity_token_program: usize,
    pub superlendy_program: usize,
    pub reserves: Vec<usize>,
    pub reserve_liquidity_supplies: Vec<usize>,
    pub lp_token_mints: Vec<usize>,
    pub lp_token_storages: Vec<usize>,
}
impl RebalanceAccountIndexes {
    pub const VAULT: usize = 0usize;
    pub const PROGRAM_AUTHORITY: usize = 1usize;
    pub const LIQUIDITY_MINT: usize = 2usize;
    pub const LIQUIDITY_BUFFER: usize = 3usize;
    pub const SUPERLENDY_PROGRAM_AUTHORITY: usize = 4usize;
    pub const LP_TOKEN_PROGRAM: usize = 5usize;
    pub const LIQUIDITY_TOKEN_PROGRAM: usize = 6usize;
    pub const SUPERLENDY_PROGRAM: usize = 7usize;
    pub fn new_direct_order(
        reserves_size: usize,
        reserve_liquidity_supplies_size: usize,
        lp_token_mints_size: usize,
        lp_token_storages_size: usize,
    ) -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            vault: iter.next().unwrap(),
            program_authority: iter.next().unwrap(),
            liquidity_mint: iter.next().unwrap(),
            liquidity_buffer: iter.next().unwrap(),
            superlendy_program_authority: iter.next().unwrap(),
            lp_token_program: iter.next().unwrap(),
            liquidity_token_program: iter.next().unwrap(),
            superlendy_program: iter.next().unwrap(),
            reserves: {
                let mut out = vec![];
                for _ in 0..reserves_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
            reserve_liquidity_supplies: {
                let mut out = vec![];
                for _ in 0..reserve_liquidity_supplies_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
            lp_token_mints: {
                let mut out = vec![];
                for _ in 0..lp_token_mints_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
            lp_token_storages: {
                let mut out = vec![];
                for _ in 0..lp_token_storages_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
        reserves_size: usize,
        reserve_liquidity_supplies_size: usize,
        lp_token_mints_size: usize,
        lp_token_storages_size: usize,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            program_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_mint: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_buffer: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            superlendy_program_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            lp_token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            superlendy_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            reserves: {
                let mut out = vec![];
                for _ in 0..reserves_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
            reserve_liquidity_supplies: {
                let mut out = vec![];
                for _ in 0..reserve_liquidity_supplies_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
            lp_token_mints: {
                let mut out = vec![];
                for _ in 0..lp_token_mints_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
            lp_token_storages: {
                let mut out = vec![];
                for _ in 0..lp_token_storages_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
        })
    }
}
/// [VaultInstruction::InitRewardSupply] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct InitRewardSupplyAccountIndexes {
    pub reward_supply: usize,
    pub reward_mint: usize,
    pub vault: usize,
    pub curator_vaults_authority: usize,
    pub curator: usize,
    pub reward_authority: usize,
    pub token_program: usize,
    pub system_program: usize,
}
impl InitRewardSupplyAccountIndexes {
    pub const COUNT: usize = 8usize;
    pub const REWARD_SUPPLY: usize = 0usize;
    pub const REWARD_MINT: usize = 1usize;
    pub const VAULT: usize = 2usize;
    pub const CURATOR_VAULTS_AUTHORITY: usize = 3usize;
    pub const CURATOR: usize = 4usize;
    pub const REWARD_AUTHORITY: usize = 5usize;
    pub const TOKEN_PROGRAM: usize = 6usize;
    pub const SYSTEM_PROGRAM: usize = 7usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            reward_supply: iter.next().unwrap(),
            reward_mint: iter.next().unwrap(),
            vault: iter.next().unwrap(),
            curator_vaults_authority: iter.next().unwrap(),
            curator: iter.next().unwrap(),
            reward_authority: iter.next().unwrap(),
            token_program: iter.next().unwrap(),
            system_program: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            reward_supply: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            reward_mint: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator_vaults_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            reward_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            system_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for InitRewardSupplyAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for InitRewardSupplyAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for InitRewardSupplyAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for InitRewardSupplyAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [VaultInstruction::SetRewardRules] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct SetRewardRulesAccountIndexes {
    pub vault: usize,
    pub curator_vaults_authority: usize,
    pub curator: usize,
}
impl SetRewardRulesAccountIndexes {
    pub const COUNT: usize = 3usize;
    pub const VAULT: usize = 0usize;
    pub const CURATOR_VAULTS_AUTHORITY: usize = 1usize;
    pub const CURATOR: usize = 2usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            vault: iter.next().unwrap(),
            curator_vaults_authority: iter.next().unwrap(),
            curator: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator_vaults_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for SetRewardRulesAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for SetRewardRulesAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for SetRewardRulesAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for SetRewardRulesAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [VaultInstruction::ClaimReward] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct ClaimRewardAccountIndexes {
    pub position: usize,
    pub rewards_supply: usize,
    pub destination_wallet: usize,
    pub position_owner: usize,
    pub vault: usize,
    pub reward_mint: usize,
    pub reward_authority: usize,
    pub token_program: usize,
}
impl ClaimRewardAccountIndexes {
    pub const COUNT: usize = 8usize;
    pub const POSITION: usize = 0usize;
    pub const REWARDS_SUPPLY: usize = 1usize;
    pub const DESTINATION_WALLET: usize = 2usize;
    pub const POSITION_OWNER: usize = 3usize;
    pub const VAULT: usize = 4usize;
    pub const REWARD_MINT: usize = 5usize;
    pub const REWARD_AUTHORITY: usize = 6usize;
    pub const TOKEN_PROGRAM: usize = 7usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            position: iter.next().unwrap(),
            rewards_supply: iter.next().unwrap(),
            destination_wallet: iter.next().unwrap(),
            position_owner: iter.next().unwrap(),
            vault: iter.next().unwrap(),
            reward_mint: iter.next().unwrap(),
            reward_authority: iter.next().unwrap(),
            token_program: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            position: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            rewards_supply: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            destination_wallet: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            position_owner: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            reward_mint: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            reward_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for ClaimRewardAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for ClaimRewardAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for ClaimRewardAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for ClaimRewardAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [VaultInstruction::WithdrawReward] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct WithdrawRewardAccountIndexes {
    pub rewards_supply: usize,
    pub destination_wallet: usize,
    pub vault: usize,
    pub curator_vaults_authority: usize,
    pub curator: usize,
    pub reward_mint: usize,
    pub reward_authority: usize,
    pub token_program: usize,
}
impl WithdrawRewardAccountIndexes {
    pub const COUNT: usize = 8usize;
    pub const REWARDS_SUPPLY: usize = 0usize;
    pub const DESTINATION_WALLET: usize = 1usize;
    pub const VAULT: usize = 2usize;
    pub const CURATOR_VAULTS_AUTHORITY: usize = 3usize;
    pub const CURATOR: usize = 4usize;
    pub const REWARD_MINT: usize = 5usize;
    pub const REWARD_AUTHORITY: usize = 6usize;
    pub const TOKEN_PROGRAM: usize = 7usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            rewards_supply: iter.next().unwrap(),
            destination_wallet: iter.next().unwrap(),
            vault: iter.next().unwrap(),
            curator_vaults_authority: iter.next().unwrap(),
            curator: iter.next().unwrap(),
            reward_mint: iter.next().unwrap(),
            reward_authority: iter.next().unwrap(),
            token_program: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            rewards_supply: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            destination_wallet: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator_vaults_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            reward_mint: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            reward_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for WithdrawRewardAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for WithdrawRewardAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for WithdrawRewardAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for WithdrawRewardAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [VaultInstruction::ClaimFees] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct ClaimFeesAccountIndexes {
    pub vault: usize,
    pub curator_vaults_authority: usize,
    pub curator: usize,
    pub liquidity_mint: usize,
    pub fee_receiver: usize,
    pub liquidity_buffer: usize,
    pub program_authority: usize,
    pub superlendy_program_authority: usize,
    pub lp_token_program: usize,
    pub liquidity_token_program: usize,
    pub superlendy_program: usize,
    pub reserves: Vec<usize>,
    pub reserve_liquidity_supplies: Vec<usize>,
    pub lp_token_mints: Vec<usize>,
    pub lp_token_storages: Vec<usize>,
}
impl ClaimFeesAccountIndexes {
    pub const VAULT: usize = 0usize;
    pub const CURATOR_VAULTS_AUTHORITY: usize = 1usize;
    pub const CURATOR: usize = 2usize;
    pub const LIQUIDITY_MINT: usize = 3usize;
    pub const FEE_RECEIVER: usize = 4usize;
    pub const LIQUIDITY_BUFFER: usize = 5usize;
    pub const PROGRAM_AUTHORITY: usize = 6usize;
    pub const SUPERLENDY_PROGRAM_AUTHORITY: usize = 7usize;
    pub const LP_TOKEN_PROGRAM: usize = 8usize;
    pub const LIQUIDITY_TOKEN_PROGRAM: usize = 9usize;
    pub const SUPERLENDY_PROGRAM: usize = 10usize;
    pub fn new_direct_order(
        reserves_size: usize,
        reserve_liquidity_supplies_size: usize,
        lp_token_mints_size: usize,
        lp_token_storages_size: usize,
    ) -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            vault: iter.next().unwrap(),
            curator_vaults_authority: iter.next().unwrap(),
            curator: iter.next().unwrap(),
            liquidity_mint: iter.next().unwrap(),
            fee_receiver: iter.next().unwrap(),
            liquidity_buffer: iter.next().unwrap(),
            program_authority: iter.next().unwrap(),
            superlendy_program_authority: iter.next().unwrap(),
            lp_token_program: iter.next().unwrap(),
            liquidity_token_program: iter.next().unwrap(),
            superlendy_program: iter.next().unwrap(),
            reserves: {
                let mut out = vec![];
                for _ in 0..reserves_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
            reserve_liquidity_supplies: {
                let mut out = vec![];
                for _ in 0..reserve_liquidity_supplies_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
            lp_token_mints: {
                let mut out = vec![];
                for _ in 0..lp_token_mints_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
            lp_token_storages: {
                let mut out = vec![];
                for _ in 0..lp_token_storages_size {
                    out.push(iter.next().unwrap());
                }
                out
            },
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
        reserves_size: usize,
        reserve_liquidity_supplies_size: usize,
        lp_token_mints_size: usize,
        lp_token_storages_size: usize,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            vault: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator_vaults_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            curator: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_mint: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            fee_receiver: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_buffer: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            program_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            superlendy_program_authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            lp_token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            liquidity_token_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            superlendy_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            reserves: {
                let mut out = vec![];
                for _ in 0..reserves_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
            reserve_liquidity_supplies: {
                let mut out = vec![];
                for _ in 0..reserve_liquidity_supplies_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
            lp_token_mints: {
                let mut out = vec![];
                for _ in 0..lp_token_mints_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
            lp_token_storages: {
                let mut out = vec![];
                for _ in 0..lp_token_storages_size {
                    idx += 1;
                    out.push(iter.next().ok_or(idx - 1)?);
                }
                out
            },
        })
    }
}
///[VaultInstruction::Version] instruction account infos helper
#[derive(Debug)]
pub struct VersionAccounts {}
impl VersionAccounts {
    pub fn from_iter<'a, 'i: 'a, I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let _ = iter;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        Ok(Self {})
    }
}
///[VaultInstruction::CreateVault] instruction account infos helper
#[derive(Debug)]
pub struct CreateVaultAccounts<'a, 'i> {
    ///Vault account to create.
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    ///Authority assigned by Curator to manage Vaults.
    pub curator_vaults_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///Curator of the Vault.
    pub curator: &'a solana_program::account_info::AccountInfo<'i>,
    pub program_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///Liquidity tokens mint
    pub liquidity_mint: &'a solana_program::account_info::AccountInfo<'i>,
    pub liquidity_buffer: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token program - classic one
    pub lp_token_program: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token program to manage liquidity tokens. Either classic or 2022
    pub liquidity_token_program: &'a solana_program::account_info::AccountInfo<'i>,
    ///System Program
    pub system_program: &'a solana_program::account_info::AccountInfo<'i>,
    pub reserves: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
    pub lp_token_mints: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
    pub lp_token_storages: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
}
impl<'a, 'i> CreateVaultAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        reserves_count: usize,
        lp_token_mints_count: usize,
        lp_token_storages_count: usize,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let vault = texture_common::utils::next_account_info(iter)?;
        let curator_vaults_authority = texture_common::utils::next_account_info(iter)?;
        let curator = texture_common::utils::next_account_info(iter)?;
        let program_authority = texture_common::utils::next_account_info(iter)?;
        let liquidity_mint = texture_common::utils::next_account_info(iter)?;
        let liquidity_buffer = texture_common::utils::next_account_info(iter)?;
        let lp_token_program = texture_common::utils::next_account_info(iter)?;
        let liquidity_token_program = texture_common::utils::next_account_info(iter)?;
        let system_program = texture_common::utils::next_account_info(iter)?;
        let reserves = {
            let mut out = vec![];
            for _ in 0..reserves_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        let lp_token_mints = {
            let mut out = vec![];
            for _ in 0..lp_token_mints_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        let lp_token_storages = {
            let mut out = vec![];
            for _ in 0..lp_token_storages_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        if !vault.is_writable {
            solana_program::msg!(concat!(stringify!(vault), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*vault.key).into());
        }
        if !vault.is_signer {
            return Err(texture_common::error::MissingSignature(*vault.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &solana_program::system_program::ID,
            concat!(stringify!(vault), " owner"),
        )?;
        if vault.data_len() != 0 {
            solana_program::msg!(
                concat!("invalid ", stringify!(vault), " account size")
            );
            return Err(texture_common::error::InvalidAccount(*vault.key).into());
        }
        if !curator_vaults_authority.is_writable {
            solana_program::msg!(
                concat!(stringify!(curator_vaults_authority), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*curator_vaults_authority.key)
                    .into(),
            );
        }
        if !curator_vaults_authority.is_signer {
            return Err(
                texture_common::error::MissingSignature(*curator_vaults_authority.key)
                    .into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            curator.owner,
            &SUPER_LENDY_ID,
            concat!(stringify!(curator), " owner"),
        )?;
        if !liquidity_buffer.is_writable {
            solana_program::msg!(
                concat!(stringify!(liquidity_buffer), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*liquidity_buffer.key).into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            lp_token_program.key,
            &spl_token::ID,
            stringify!(lp_token_program),
        )?;
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            system_program.key,
            &solana_program::system_program::ID,
            stringify!(system_program),
        )?;
        for acc in &reserves {
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &SUPER_LENDY_ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        for acc in &lp_token_mints {
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &spl_token::ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        for acc in &lp_token_storages {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &solana_program::system_program::ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        Ok(Self {
            vault,
            curator_vaults_authority,
            curator,
            program_authority,
            liquidity_mint,
            liquidity_buffer,
            lp_token_program,
            liquidity_token_program,
            system_program,
            reserves,
            lp_token_mints,
            lp_token_storages,
        })
    }
}
///[VaultInstruction::AlterVault] instruction account infos helper
#[derive(Debug)]
pub struct AlterVaultAccounts<'a, 'i> {
    ///Vault account to change.
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    ///Authority assigned by Curator to manage Vaults.
    pub curator_vaults_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///Authority who can change percentage in distribution.
    pub distribution_manager: &'a solana_program::account_info::AccountInfo<'i>,
    ///Curator of the Vault.
    pub curator: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> AlterVaultAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let vault = texture_common::utils::next_account_info(iter)?;
        let curator_vaults_authority = texture_common::utils::next_account_info(iter)?;
        let distribution_manager = texture_common::utils::next_account_info(iter)?;
        let curator = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        if !vault.is_writable {
            solana_program::msg!(concat!(stringify!(vault), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*vault.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &__self_program_id__,
            concat!(stringify!(vault), " owner"),
        )?;
        if !curator_vaults_authority.is_writable {
            solana_program::msg!(
                concat!(stringify!(curator_vaults_authority), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*curator_vaults_authority.key)
                    .into(),
            );
        }
        if !distribution_manager.is_writable {
            solana_program::msg!(
                concat!(stringify!(distribution_manager), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*distribution_manager.key).into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            curator.owner,
            &SUPER_LENDY_ID,
            concat!(stringify!(curator), " owner"),
        )?;
        Ok(Self {
            vault,
            curator_vaults_authority,
            distribution_manager,
            curator,
        })
    }
}
///[VaultInstruction::ProposeDistributionItem] instruction account infos helper
#[derive(Debug)]
pub struct ProposeDistributionItemAccounts<'a, 'i> {
    ///Vault account to change.
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    ///Authority assigned by Curator to manage Vaults.
    pub curator_vaults_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///Curator of the Vault.
    pub curator: &'a solana_program::account_info::AccountInfo<'i>,
    ///Reserve to be added in to distribution
    pub reserve: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> ProposeDistributionItemAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let vault = texture_common::utils::next_account_info(iter)?;
        let curator_vaults_authority = texture_common::utils::next_account_info(iter)?;
        let curator = texture_common::utils::next_account_info(iter)?;
        let reserve = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        if !vault.is_writable {
            solana_program::msg!(concat!(stringify!(vault), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*vault.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &__self_program_id__,
            concat!(stringify!(vault), " owner"),
        )?;
        if !curator_vaults_authority.is_writable {
            solana_program::msg!(
                concat!(stringify!(curator_vaults_authority), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*curator_vaults_authority.key)
                    .into(),
            );
        }
        if !curator_vaults_authority.is_signer {
            return Err(
                texture_common::error::MissingSignature(*curator_vaults_authority.key)
                    .into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            curator.owner,
            &SUPER_LENDY_ID,
            concat!(stringify!(curator), " owner"),
        )?;
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            reserve.owner,
            &SUPER_LENDY_ID,
            concat!(stringify!(reserve), " owner"),
        )?;
        Ok(Self {
            vault,
            curator_vaults_authority,
            curator,
            reserve,
        })
    }
}
///[VaultInstruction::AddDistributionItem] instruction account infos helper
#[derive(Debug)]
pub struct AddDistributionItemAccounts<'a, 'i> {
    ///Vault account to change.
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    ///Authority assigned by Curator to manage Vaults.
    pub curator_vaults_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///Curator of the Vault.
    pub curator: &'a solana_program::account_info::AccountInfo<'i>,
    ///Mint of the reserve's LP tokens
    pub lp_token_mint: &'a solana_program::account_info::AccountInfo<'i>,
    ///LP token storage to be used by Vault. Unitialized.
    pub lp_token_storage: &'a solana_program::account_info::AccountInfo<'i>,
    pub program_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token Program
    pub token_program: &'a solana_program::account_info::AccountInfo<'i>,
    ///System Program
    pub system_program: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> AddDistributionItemAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let vault = texture_common::utils::next_account_info(iter)?;
        let curator_vaults_authority = texture_common::utils::next_account_info(iter)?;
        let curator = texture_common::utils::next_account_info(iter)?;
        let lp_token_mint = texture_common::utils::next_account_info(iter)?;
        let lp_token_storage = texture_common::utils::next_account_info(iter)?;
        let program_authority = texture_common::utils::next_account_info(iter)?;
        let token_program = texture_common::utils::next_account_info(iter)?;
        let system_program = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        if !vault.is_writable {
            solana_program::msg!(concat!(stringify!(vault), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*vault.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &__self_program_id__,
            concat!(stringify!(vault), " owner"),
        )?;
        if !curator_vaults_authority.is_writable {
            solana_program::msg!(
                concat!(stringify!(curator_vaults_authority), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*curator_vaults_authority.key)
                    .into(),
            );
        }
        if !curator_vaults_authority.is_signer {
            return Err(
                texture_common::error::MissingSignature(*curator_vaults_authority.key)
                    .into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            curator.owner,
            &SUPER_LENDY_ID,
            concat!(stringify!(curator), " owner"),
        )?;
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            lp_token_mint.owner,
            &spl_token::ID,
            concat!(stringify!(lp_token_mint), " owner"),
        )?;
        if !lp_token_storage.is_writable {
            solana_program::msg!(
                concat!(stringify!(lp_token_storage), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*lp_token_storage.key).into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            lp_token_storage.owner,
            &solana_program::system_program::ID,
            concat!(stringify!(lp_token_storage), " owner"),
        )?;
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            token_program.key,
            &spl_token::ID,
            stringify!(token_program),
        )?;
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            system_program.key,
            &solana_program::system_program::ID,
            stringify!(system_program),
        )?;
        Ok(Self {
            vault,
            curator_vaults_authority,
            curator,
            lp_token_mint,
            lp_token_storage,
            program_authority,
            token_program,
            system_program,
        })
    }
}
///[VaultInstruction::RemoveDistributionItem] instruction account infos helper
#[derive(Debug)]
pub struct RemoveDistributionItemAccounts<'a, 'i> {
    ///Vault account to change.
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    ///Authority assigned by Curator to manage Vaults.
    pub curator_vaults_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///Curator of the Vault.
    pub curator: &'a solana_program::account_info::AccountInfo<'i>,
    ///LP token storage.
    pub lp_token_storage: &'a solana_program::account_info::AccountInfo<'i>,
    pub program_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token Program
    pub token_program: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> RemoveDistributionItemAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let vault = texture_common::utils::next_account_info(iter)?;
        let curator_vaults_authority = texture_common::utils::next_account_info(iter)?;
        let curator = texture_common::utils::next_account_info(iter)?;
        let lp_token_storage = texture_common::utils::next_account_info(iter)?;
        let program_authority = texture_common::utils::next_account_info(iter)?;
        let token_program = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        if !vault.is_writable {
            solana_program::msg!(concat!(stringify!(vault), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*vault.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &__self_program_id__,
            concat!(stringify!(vault), " owner"),
        )?;
        if !curator_vaults_authority.is_writable {
            solana_program::msg!(
                concat!(stringify!(curator_vaults_authority), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*curator_vaults_authority.key)
                    .into(),
            );
        }
        if !curator_vaults_authority.is_signer {
            return Err(
                texture_common::error::MissingSignature(*curator_vaults_authority.key)
                    .into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            curator.owner,
            &SUPER_LENDY_ID,
            concat!(stringify!(curator), " owner"),
        )?;
        if !lp_token_storage.is_writable {
            solana_program::msg!(
                concat!(stringify!(lp_token_storage), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*lp_token_storage.key).into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            lp_token_storage.owner,
            &spl_token::ID,
            concat!(stringify!(lp_token_storage), " owner"),
        )?;
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            token_program.key,
            &spl_token::ID,
            stringify!(token_program),
        )?;
        Ok(Self {
            vault,
            curator_vaults_authority,
            curator,
            lp_token_storage,
            program_authority,
            token_program,
        })
    }
}
///[VaultInstruction::CreatePosition] instruction account infos helper
#[derive(Debug)]
pub struct CreatePositionAccounts<'a, 'i> {
    ///Position account to create.
    pub position: &'a solana_program::account_info::AccountInfo<'i>,
    ///Vault account the position belongs to.
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    ///Position owner.
    pub owner: &'a solana_program::account_info::AccountInfo<'i>,
    pub program_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///System Program
    pub system_program: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> CreatePositionAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let position = texture_common::utils::next_account_info(iter)?;
        let vault = texture_common::utils::next_account_info(iter)?;
        let owner = texture_common::utils::next_account_info(iter)?;
        let program_authority = texture_common::utils::next_account_info(iter)?;
        let system_program = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        if !position.is_writable {
            solana_program::msg!(concat!(stringify!(position), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*position.key).into());
        }
        if !position.is_signer {
            return Err(texture_common::error::MissingSignature(*position.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            position.owner,
            &solana_program::system_program::ID,
            concat!(stringify!(position), " owner"),
        )?;
        if position.data_len() != 0 {
            solana_program::msg!(
                concat!("invalid ", stringify!(position), " account size")
            );
            return Err(texture_common::error::InvalidAccount(*position.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &__self_program_id__,
            concat!(stringify!(vault), " owner"),
        )?;
        if !owner.is_writable {
            solana_program::msg!(concat!(stringify!(owner), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*owner.key).into());
        }
        if !owner.is_signer {
            return Err(texture_common::error::MissingSignature(*owner.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            system_program.key,
            &solana_program::system_program::ID,
            stringify!(system_program),
        )?;
        Ok(Self {
            position,
            vault,
            owner,
            program_authority,
            system_program,
        })
    }
}
///[VaultInstruction::Deposit] instruction account infos helper
#[derive(Debug)]
pub struct DepositAccounts<'a, 'i> {
    ///Vault account to deposit to.
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    ///Position account to deposit thru.
    pub position: &'a solana_program::account_info::AccountInfo<'i>,
    ///Position owner.
    pub owner: &'a solana_program::account_info::AccountInfo<'i>,
    ///Liquidity tokens mint
    pub liquidity_mint: &'a solana_program::account_info::AccountInfo<'i>,
    pub liquidity_source: &'a solana_program::account_info::AccountInfo<'i>,
    pub liquidity_buffer: &'a solana_program::account_info::AccountInfo<'i>,
    pub program_authority: &'a solana_program::account_info::AccountInfo<'i>,
    pub superlendy_program_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token program - classic one
    pub lp_token_program: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token program to manage liquidity tokens. Either classic or 2022
    pub liquidity_token_program: &'a solana_program::account_info::AccountInfo<'i>,
    ///Superlendy Program
    pub superlendy_program: &'a solana_program::account_info::AccountInfo<'i>,
    pub reserves: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
    pub reserve_liquidity_supplies: Vec<
        &'a solana_program::account_info::AccountInfo<'i>,
    >,
    pub lp_token_mints: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
    pub lp_token_storages: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
}
impl<'a, 'i> DepositAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        reserves_count: usize,
        reserve_liquidity_supplies_count: usize,
        lp_token_mints_count: usize,
        lp_token_storages_count: usize,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let vault = texture_common::utils::next_account_info(iter)?;
        let position = texture_common::utils::next_account_info(iter)?;
        let owner = texture_common::utils::next_account_info(iter)?;
        let liquidity_mint = texture_common::utils::next_account_info(iter)?;
        let liquidity_source = texture_common::utils::next_account_info(iter)?;
        let liquidity_buffer = texture_common::utils::next_account_info(iter)?;
        let program_authority = texture_common::utils::next_account_info(iter)?;
        let superlendy_program_authority = texture_common::utils::next_account_info(
            iter,
        )?;
        let lp_token_program = texture_common::utils::next_account_info(iter)?;
        let liquidity_token_program = texture_common::utils::next_account_info(iter)?;
        let superlendy_program = texture_common::utils::next_account_info(iter)?;
        let reserves = {
            let mut out = vec![];
            for _ in 0..reserves_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        let reserve_liquidity_supplies = {
            let mut out = vec![];
            for _ in 0..reserve_liquidity_supplies_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        let lp_token_mints = {
            let mut out = vec![];
            for _ in 0..lp_token_mints_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        let lp_token_storages = {
            let mut out = vec![];
            for _ in 0..lp_token_storages_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        if !vault.is_writable {
            solana_program::msg!(concat!(stringify!(vault), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*vault.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &__self_program_id__,
            concat!(stringify!(vault), " owner"),
        )?;
        if !position.is_writable {
            solana_program::msg!(concat!(stringify!(position), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*position.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            position.owner,
            &__self_program_id__,
            concat!(stringify!(position), " owner"),
        )?;
        if !owner.is_writable {
            solana_program::msg!(concat!(stringify!(owner), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*owner.key).into());
        }
        if !owner.is_signer {
            return Err(texture_common::error::MissingSignature(*owner.key).into());
        }
        if !liquidity_source.is_writable {
            solana_program::msg!(
                concat!(stringify!(liquidity_source), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*liquidity_source.key).into(),
            );
        }
        if !liquidity_buffer.is_writable {
            solana_program::msg!(
                concat!(stringify!(liquidity_buffer), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*liquidity_buffer.key).into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            lp_token_program.key,
            &spl_token::ID,
            stringify!(lp_token_program),
        )?;
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            superlendy_program.key,
            &super_lendy::SUPER_LENDY_ID,
            stringify!(superlendy_program),
        )?;
        for acc in &reserves {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &SUPER_LENDY_ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        for acc in &reserve_liquidity_supplies {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
        }
        for acc in &lp_token_mints {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &spl_token::ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        for acc in &lp_token_storages {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &spl_token::ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        Ok(Self {
            vault,
            position,
            owner,
            liquidity_mint,
            liquidity_source,
            liquidity_buffer,
            program_authority,
            superlendy_program_authority,
            lp_token_program,
            liquidity_token_program,
            superlendy_program,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
        })
    }
}
///[VaultInstruction::Withdraw] instruction account infos helper
#[derive(Debug)]
pub struct WithdrawAccounts<'a, 'i> {
    ///Vault account to withdraw from.
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    ///Position account to withdraw from.
    pub position: &'a solana_program::account_info::AccountInfo<'i>,
    ///Position owner.
    pub owner: &'a solana_program::account_info::AccountInfo<'i>,
    ///Liquidity tokens mint
    pub liquidity_mint: &'a solana_program::account_info::AccountInfo<'i>,
    pub liquidity_destination: &'a solana_program::account_info::AccountInfo<'i>,
    pub liquidity_buffer: &'a solana_program::account_info::AccountInfo<'i>,
    pub program_authority: &'a solana_program::account_info::AccountInfo<'i>,
    pub superlendy_program_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token program - classic one
    pub lp_token_program: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token program to manage liquidity tokens. Either classic or 2022
    pub liquidity_token_program: &'a solana_program::account_info::AccountInfo<'i>,
    ///Superlendy Program
    pub superlendy_program: &'a solana_program::account_info::AccountInfo<'i>,
    pub reserves: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
    pub reserve_liquidity_supplies: Vec<
        &'a solana_program::account_info::AccountInfo<'i>,
    >,
    pub lp_token_mints: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
    pub lp_token_storages: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
}
impl<'a, 'i> WithdrawAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        reserves_count: usize,
        reserve_liquidity_supplies_count: usize,
        lp_token_mints_count: usize,
        lp_token_storages_count: usize,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let vault = texture_common::utils::next_account_info(iter)?;
        let position = texture_common::utils::next_account_info(iter)?;
        let owner = texture_common::utils::next_account_info(iter)?;
        let liquidity_mint = texture_common::utils::next_account_info(iter)?;
        let liquidity_destination = texture_common::utils::next_account_info(iter)?;
        let liquidity_buffer = texture_common::utils::next_account_info(iter)?;
        let program_authority = texture_common::utils::next_account_info(iter)?;
        let superlendy_program_authority = texture_common::utils::next_account_info(
            iter,
        )?;
        let lp_token_program = texture_common::utils::next_account_info(iter)?;
        let liquidity_token_program = texture_common::utils::next_account_info(iter)?;
        let superlendy_program = texture_common::utils::next_account_info(iter)?;
        let reserves = {
            let mut out = vec![];
            for _ in 0..reserves_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        let reserve_liquidity_supplies = {
            let mut out = vec![];
            for _ in 0..reserve_liquidity_supplies_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        let lp_token_mints = {
            let mut out = vec![];
            for _ in 0..lp_token_mints_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        let lp_token_storages = {
            let mut out = vec![];
            for _ in 0..lp_token_storages_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        if !vault.is_writable {
            solana_program::msg!(concat!(stringify!(vault), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*vault.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &__self_program_id__,
            concat!(stringify!(vault), " owner"),
        )?;
        if !position.is_writable {
            solana_program::msg!(concat!(stringify!(position), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*position.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            position.owner,
            &__self_program_id__,
            concat!(stringify!(position), " owner"),
        )?;
        if !owner.is_writable {
            solana_program::msg!(concat!(stringify!(owner), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*owner.key).into());
        }
        if !owner.is_signer {
            return Err(texture_common::error::MissingSignature(*owner.key).into());
        }
        if !liquidity_destination.is_writable {
            solana_program::msg!(
                concat!(stringify!(liquidity_destination), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*liquidity_destination.key).into(),
            );
        }
        if !liquidity_buffer.is_writable {
            solana_program::msg!(
                concat!(stringify!(liquidity_buffer), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*liquidity_buffer.key).into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            lp_token_program.key,
            &spl_token::ID,
            stringify!(lp_token_program),
        )?;
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            superlendy_program.key,
            &super_lendy::SUPER_LENDY_ID,
            stringify!(superlendy_program),
        )?;
        for acc in &reserves {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &SUPER_LENDY_ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        for acc in &reserve_liquidity_supplies {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
        }
        for acc in &lp_token_mints {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &spl_token::ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        for acc in &lp_token_storages {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &spl_token::ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        Ok(Self {
            vault,
            position,
            owner,
            liquidity_mint,
            liquidity_destination,
            liquidity_buffer,
            program_authority,
            superlendy_program_authority,
            lp_token_program,
            liquidity_token_program,
            superlendy_program,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
        })
    }
}
///[VaultInstruction::ClosePosition] instruction account infos helper
#[derive(Debug)]
pub struct ClosePositionAccounts<'a, 'i> {
    ///Position account to close.
    pub position: &'a solana_program::account_info::AccountInfo<'i>,
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    ///Owner of the position
    pub owner: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> ClosePositionAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let position = texture_common::utils::next_account_info(iter)?;
        let vault = texture_common::utils::next_account_info(iter)?;
        let owner = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        if !position.is_writable {
            solana_program::msg!(concat!(stringify!(position), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*position.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            position.owner,
            &__self_program_id__,
            concat!(stringify!(position), " owner"),
        )?;
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &__self_program_id__,
            concat!(stringify!(vault), " owner"),
        )?;
        if !owner.is_writable {
            solana_program::msg!(concat!(stringify!(owner), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*owner.key).into());
        }
        if !owner.is_signer {
            return Err(texture_common::error::MissingSignature(*owner.key).into());
        }
        Ok(Self { position, vault, owner })
    }
}
///[VaultInstruction::Rebalance] instruction account infos helper
#[derive(Debug)]
pub struct RebalanceAccounts<'a, 'i> {
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    pub program_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///Liquidity tokens mint
    pub liquidity_mint: &'a solana_program::account_info::AccountInfo<'i>,
    pub liquidity_buffer: &'a solana_program::account_info::AccountInfo<'i>,
    pub superlendy_program_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token program - classic one
    pub lp_token_program: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token program to manage liquidity tokens. Either classic or 2022
    pub liquidity_token_program: &'a solana_program::account_info::AccountInfo<'i>,
    ///Superlendy Program
    pub superlendy_program: &'a solana_program::account_info::AccountInfo<'i>,
    pub reserves: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
    pub reserve_liquidity_supplies: Vec<
        &'a solana_program::account_info::AccountInfo<'i>,
    >,
    pub lp_token_mints: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
    pub lp_token_storages: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
}
impl<'a, 'i> RebalanceAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        reserves_count: usize,
        reserve_liquidity_supplies_count: usize,
        lp_token_mints_count: usize,
        lp_token_storages_count: usize,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let vault = texture_common::utils::next_account_info(iter)?;
        let program_authority = texture_common::utils::next_account_info(iter)?;
        let liquidity_mint = texture_common::utils::next_account_info(iter)?;
        let liquidity_buffer = texture_common::utils::next_account_info(iter)?;
        let superlendy_program_authority = texture_common::utils::next_account_info(
            iter,
        )?;
        let lp_token_program = texture_common::utils::next_account_info(iter)?;
        let liquidity_token_program = texture_common::utils::next_account_info(iter)?;
        let superlendy_program = texture_common::utils::next_account_info(iter)?;
        let reserves = {
            let mut out = vec![];
            for _ in 0..reserves_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        let reserve_liquidity_supplies = {
            let mut out = vec![];
            for _ in 0..reserve_liquidity_supplies_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        let lp_token_mints = {
            let mut out = vec![];
            for _ in 0..lp_token_mints_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        let lp_token_storages = {
            let mut out = vec![];
            for _ in 0..lp_token_storages_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &__self_program_id__,
            concat!(stringify!(vault), " owner"),
        )?;
        if !liquidity_buffer.is_writable {
            solana_program::msg!(
                concat!(stringify!(liquidity_buffer), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*liquidity_buffer.key).into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            lp_token_program.key,
            &spl_token::ID,
            stringify!(lp_token_program),
        )?;
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            superlendy_program.key,
            &super_lendy::SUPER_LENDY_ID,
            stringify!(superlendy_program),
        )?;
        for acc in &reserves {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &SUPER_LENDY_ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        for acc in &reserve_liquidity_supplies {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
        }
        for acc in &lp_token_mints {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &spl_token::ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        for acc in &lp_token_storages {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &spl_token::ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        Ok(Self {
            vault,
            program_authority,
            liquidity_mint,
            liquidity_buffer,
            superlendy_program_authority,
            lp_token_program,
            liquidity_token_program,
            superlendy_program,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
        })
    }
}
///[VaultInstruction::InitRewardSupply] instruction account infos helper
#[derive(Debug)]
pub struct InitRewardSupplyAccounts<'a, 'i> {
    ///Reward supply account to initialize. Uninitialized. PDA
    pub reward_supply: &'a solana_program::account_info::AccountInfo<'i>,
    ///Reward token mint.
    pub reward_mint: &'a solana_program::account_info::AccountInfo<'i>,
    ///Vault to init reward supply for.
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    ///Authority who can manage Vault.
    pub curator_vaults_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///Curator account.
    pub curator: &'a solana_program::account_info::AccountInfo<'i>,
    ///Contract's reward authority. PDA.
    pub reward_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token program
    pub token_program: &'a solana_program::account_info::AccountInfo<'i>,
    ///System Program.
    pub system_program: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> InitRewardSupplyAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let reward_supply = texture_common::utils::next_account_info(iter)?;
        let reward_mint = texture_common::utils::next_account_info(iter)?;
        let vault = texture_common::utils::next_account_info(iter)?;
        let curator_vaults_authority = texture_common::utils::next_account_info(iter)?;
        let curator = texture_common::utils::next_account_info(iter)?;
        let reward_authority = texture_common::utils::next_account_info(iter)?;
        let token_program = texture_common::utils::next_account_info(iter)?;
        let system_program = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        let rent = <solana_program::rent::Rent as solana_program::sysvar::Sysvar>::get()
            .expect("rent");
        if !reward_supply.is_writable {
            solana_program::msg!(concat!(stringify!(reward_supply), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*reward_supply.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            reward_supply.owner,
            &solana_program::system_program::ID,
            concat!(stringify!(reward_supply), " owner"),
        )?;
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &__self_program_id__,
            concat!(stringify!(vault), " owner"),
        )?;
        if !curator_vaults_authority.is_writable {
            solana_program::msg!(
                concat!(stringify!(curator_vaults_authority), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*curator_vaults_authority.key)
                    .into(),
            );
        }
        if !curator_vaults_authority.is_signer {
            return Err(
                texture_common::error::MissingSignature(*curator_vaults_authority.key)
                    .into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            curator.owner,
            &SUPER_LENDY_ID,
            concat!(stringify!(curator), " owner"),
        )?;
        if !rent.is_exempt(curator.lamports(), curator.data_len()) {
            solana_program::msg!(concat!(stringify!(curator), " is not rent exempt"));
            return Err(texture_common::error::InvalidAccount(*curator.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            system_program.key,
            &solana_program::system_program::ID,
            stringify!(system_program),
        )?;
        Ok(Self {
            reward_supply,
            reward_mint,
            vault,
            curator_vaults_authority,
            curator,
            reward_authority,
            token_program,
            system_program,
        })
    }
}
///[VaultInstruction::SetRewardRules] instruction account infos helper
#[derive(Debug)]
pub struct SetRewardRulesAccounts<'a, 'i> {
    ///Vault account to set reward rules for
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    ///Authority who can configure Vault.
    pub curator_vaults_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///Curator account.
    pub curator: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> SetRewardRulesAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let vault = texture_common::utils::next_account_info(iter)?;
        let curator_vaults_authority = texture_common::utils::next_account_info(iter)?;
        let curator = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        let rent = <solana_program::rent::Rent as solana_program::sysvar::Sysvar>::get()
            .expect("rent");
        if !vault.is_writable {
            solana_program::msg!(concat!(stringify!(vault), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*vault.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &__self_program_id__,
            concat!(stringify!(vault), " owner"),
        )?;
        if !curator_vaults_authority.is_writable {
            solana_program::msg!(
                concat!(stringify!(curator_vaults_authority), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*curator_vaults_authority.key)
                    .into(),
            );
        }
        if !curator_vaults_authority.is_signer {
            return Err(
                texture_common::error::MissingSignature(*curator_vaults_authority.key)
                    .into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            curator.owner,
            &SUPER_LENDY_ID,
            concat!(stringify!(curator), " owner"),
        )?;
        if !rent.is_exempt(curator.lamports(), curator.data_len()) {
            solana_program::msg!(concat!(stringify!(curator), " is not rent exempt"));
            return Err(texture_common::error::InvalidAccount(*curator.key).into());
        }
        Ok(Self {
            vault,
            curator_vaults_authority,
            curator,
        })
    }
}
///[VaultInstruction::ClaimReward] instruction account infos helper
#[derive(Debug)]
pub struct ClaimRewardAccounts<'a, 'i> {
    ///Position account to claim rewords for. Refreshed.
    pub position: &'a solana_program::account_info::AccountInfo<'i>,
    ///Contract managed SPL token wallet which holds reward tokens. PDA.
    pub rewards_supply: &'a solana_program::account_info::AccountInfo<'i>,
    ///User's SPL token wallet which will receive reward tokens
    pub destination_wallet: &'a solana_program::account_info::AccountInfo<'i>,
    ///Position owner
    pub position_owner: &'a solana_program::account_info::AccountInfo<'i>,
    ///Vault, position belongs to.
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    ///Reward token mint to claim. Determines which reward will be claimed from the Position.
    pub reward_mint: &'a solana_program::account_info::AccountInfo<'i>,
    ///Contract's reward authority. PDA.
    pub reward_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token program
    pub token_program: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> ClaimRewardAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let position = texture_common::utils::next_account_info(iter)?;
        let rewards_supply = texture_common::utils::next_account_info(iter)?;
        let destination_wallet = texture_common::utils::next_account_info(iter)?;
        let position_owner = texture_common::utils::next_account_info(iter)?;
        let vault = texture_common::utils::next_account_info(iter)?;
        let reward_mint = texture_common::utils::next_account_info(iter)?;
        let reward_authority = texture_common::utils::next_account_info(iter)?;
        let token_program = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        if !position.is_writable {
            solana_program::msg!(concat!(stringify!(position), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*position.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            position.owner,
            &__self_program_id__,
            concat!(stringify!(position), " owner"),
        )?;
        if !rewards_supply.is_writable {
            solana_program::msg!(
                concat!(stringify!(rewards_supply), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*rewards_supply.key).into(),
            );
        }
        if !destination_wallet.is_writable {
            solana_program::msg!(
                concat!(stringify!(destination_wallet), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*destination_wallet.key).into(),
            );
        }
        if !position_owner.is_signer {
            return Err(
                texture_common::error::MissingSignature(*position_owner.key).into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &__self_program_id__,
            concat!(stringify!(vault), " owner"),
        )?;
        Ok(Self {
            position,
            rewards_supply,
            destination_wallet,
            position_owner,
            vault,
            reward_mint,
            reward_authority,
            token_program,
        })
    }
}
///[VaultInstruction::WithdrawReward] instruction account infos helper
#[derive(Debug)]
pub struct WithdrawRewardAccounts<'a, 'i> {
    ///Contract managed SPL token wallet which holds reward tokens. PDA.
    pub rewards_supply: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL token account of reward_mint to receive reward tokens.
    pub destination_wallet: &'a solana_program::account_info::AccountInfo<'i>,
    ///Vault to withdraw rewards from.
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    ///Authority who can configure Vault.
    pub curator_vaults_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///Curator account.
    pub curator: &'a solana_program::account_info::AccountInfo<'i>,
    ///Reward token mint to withdraw.
    pub reward_mint: &'a solana_program::account_info::AccountInfo<'i>,
    ///Contract's reward authority. PDA.
    pub reward_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token program
    pub token_program: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> WithdrawRewardAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let rewards_supply = texture_common::utils::next_account_info(iter)?;
        let destination_wallet = texture_common::utils::next_account_info(iter)?;
        let vault = texture_common::utils::next_account_info(iter)?;
        let curator_vaults_authority = texture_common::utils::next_account_info(iter)?;
        let curator = texture_common::utils::next_account_info(iter)?;
        let reward_mint = texture_common::utils::next_account_info(iter)?;
        let reward_authority = texture_common::utils::next_account_info(iter)?;
        let token_program = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        let rent = <solana_program::rent::Rent as solana_program::sysvar::Sysvar>::get()
            .expect("rent");
        if !rewards_supply.is_writable {
            solana_program::msg!(
                concat!(stringify!(rewards_supply), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*rewards_supply.key).into(),
            );
        }
        if !destination_wallet.is_writable {
            solana_program::msg!(
                concat!(stringify!(destination_wallet), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*destination_wallet.key).into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &__self_program_id__,
            concat!(stringify!(vault), " owner"),
        )?;
        if !curator_vaults_authority.is_writable {
            solana_program::msg!(
                concat!(stringify!(curator_vaults_authority), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*curator_vaults_authority.key)
                    .into(),
            );
        }
        if !curator_vaults_authority.is_signer {
            return Err(
                texture_common::error::MissingSignature(*curator_vaults_authority.key)
                    .into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            curator.owner,
            &__self_program_id__,
            concat!(stringify!(curator), " owner"),
        )?;
        if !rent.is_exempt(curator.lamports(), curator.data_len()) {
            solana_program::msg!(concat!(stringify!(curator), " is not rent exempt"));
            return Err(texture_common::error::InvalidAccount(*curator.key).into());
        }
        Ok(Self {
            rewards_supply,
            destination_wallet,
            vault,
            curator_vaults_authority,
            curator,
            reward_mint,
            reward_authority,
            token_program,
        })
    }
}
///[VaultInstruction::ClaimFees] instruction account infos helper
#[derive(Debug)]
pub struct ClaimFeesAccounts<'a, 'i> {
    ///Vault account to claim fees from.
    pub vault: &'a solana_program::account_info::AccountInfo<'i>,
    ///Authority assigned by Curator to manage Vaults.
    pub curator_vaults_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///Curator of the Vault.
    pub curator: &'a solana_program::account_info::AccountInfo<'i>,
    ///Liquidity tokens mint
    pub liquidity_mint: &'a solana_program::account_info::AccountInfo<'i>,
    ///Any SPL account of proper mint to transfer fees to
    pub fee_receiver: &'a solana_program::account_info::AccountInfo<'i>,
    ///Any SPL account of proper mint to transfer fees to
    pub liquidity_buffer: &'a solana_program::account_info::AccountInfo<'i>,
    pub program_authority: &'a solana_program::account_info::AccountInfo<'i>,
    pub superlendy_program_authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token program - classic one
    pub lp_token_program: &'a solana_program::account_info::AccountInfo<'i>,
    ///SPL Token program to manage liquidity tokens. Either classic or 2022
    pub liquidity_token_program: &'a solana_program::account_info::AccountInfo<'i>,
    ///Superlendy Program
    pub superlendy_program: &'a solana_program::account_info::AccountInfo<'i>,
    pub reserves: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
    pub reserve_liquidity_supplies: Vec<
        &'a solana_program::account_info::AccountInfo<'i>,
    >,
    pub lp_token_mints: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
    pub lp_token_storages: Vec<&'a solana_program::account_info::AccountInfo<'i>>,
}
impl<'a, 'i> ClaimFeesAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        reserves_count: usize,
        reserve_liquidity_supplies_count: usize,
        lp_token_mints_count: usize,
        lp_token_storages_count: usize,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let vault = texture_common::utils::next_account_info(iter)?;
        let curator_vaults_authority = texture_common::utils::next_account_info(iter)?;
        let curator = texture_common::utils::next_account_info(iter)?;
        let liquidity_mint = texture_common::utils::next_account_info(iter)?;
        let fee_receiver = texture_common::utils::next_account_info(iter)?;
        let liquidity_buffer = texture_common::utils::next_account_info(iter)?;
        let program_authority = texture_common::utils::next_account_info(iter)?;
        let superlendy_program_authority = texture_common::utils::next_account_info(
            iter,
        )?;
        let lp_token_program = texture_common::utils::next_account_info(iter)?;
        let liquidity_token_program = texture_common::utils::next_account_info(iter)?;
        let superlendy_program = texture_common::utils::next_account_info(iter)?;
        let reserves = {
            let mut out = vec![];
            for _ in 0..reserves_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        let reserve_liquidity_supplies = {
            let mut out = vec![];
            for _ in 0..reserve_liquidity_supplies_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        let lp_token_mints = {
            let mut out = vec![];
            for _ in 0..lp_token_mints_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        let lp_token_storages = {
            let mut out = vec![];
            for _ in 0..lp_token_storages_count {
                out.push(texture_common::utils::next_account_info(iter)?);
            }
            out
        };
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &VAULT_ID,
            "self_program_id",
        )?;
        if !vault.is_writable {
            solana_program::msg!(concat!(stringify!(vault), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*vault.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            vault.owner,
            &__self_program_id__,
            concat!(stringify!(vault), " owner"),
        )?;
        if !curator_vaults_authority.is_signer {
            return Err(
                texture_common::error::MissingSignature(*curator_vaults_authority.key)
                    .into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            curator.owner,
            &SUPER_LENDY_ID,
            concat!(stringify!(curator), " owner"),
        )?;
        if !fee_receiver.is_writable {
            solana_program::msg!(concat!(stringify!(fee_receiver), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*fee_receiver.key).into());
        }
        if !liquidity_buffer.is_writable {
            solana_program::msg!(
                concat!(stringify!(liquidity_buffer), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*liquidity_buffer.key).into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            lp_token_program.key,
            &spl_token::ID,
            stringify!(lp_token_program),
        )?;
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            superlendy_program.key,
            &super_lendy::SUPER_LENDY_ID,
            stringify!(superlendy_program),
        )?;
        for acc in &reserves {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &SUPER_LENDY_ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        for acc in &reserve_liquidity_supplies {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
        }
        for acc in &lp_token_mints {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &spl_token::ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        for acc in &lp_token_storages {
            if !acc.is_writable {
                solana_program::msg!(concat!(stringify!(acc), " is not writable"));
                return Err(texture_common::error::InvalidAccount(*acc.key).into());
            }
            #[allow(clippy::needless_borrow)]
            texture_common::utils::verify_key(
                acc.owner,
                &spl_token::ID,
                concat!(stringify!(acc), " owner"),
            )?;
        }
        Ok(Self {
            vault,
            curator_vaults_authority,
            curator,
            liquidity_mint,
            fee_receiver,
            liquidity_buffer,
            program_authority,
            superlendy_program_authority,
            lp_token_program,
            liquidity_token_program,
            superlendy_program,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
        })
    }
}
pub(crate) mod ix_docs {
    macro_rules! version {
        () => {
            concat! { " ## Accounts", "\n", " ", "Not required", "\n", " ## Usage", "\n",
            " ", "For create instruction use builder struct [Version]", " ",
            "(method [into_instruction][Version::into_instruction]).", " ", "\n\n", " ",
            "For parse accounts infos from processor use struct [VersionAccounts]", " ",
            "(method [from_iter][VersionAccounts::from_iter]).", " ", "\n\n", " ",
            "For work with account indexes use struct [VersionAccountIndexes].", "\n", }
        };
    }
    pub(crate) use version;
    macro_rules! create_vault {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable, signer\\]", "</b> ", "Vault account to create.", "\n", " ",
            "\n", "<b><i>", "1", "</i></b>. <b>", "\\[writable, signer\\]", "</b> ",
            "Authority assigned by Curator to manage Vaults.", "\n", " ", "\n", "<b><i>",
            "2", "</i></b>. <b>", "\\[\\]", "</b> ", "Curator of the Vault.", "\n", " ",
            "\n", "<b><i>", "3", "</i></b>. <b>", "\\[\\]", "</b> ",
            "`program_authority`", "\n", " ", "\n", "<b><i>", "4", "</i></b>. <b>",
            "\\[\\]", "</b> ", "Liquidity tokens mint", "\n", " ", "\n", "<b><i>", "5",
            "</i></b>. <b>", "\\[writable\\]", "</b> ", "`liquidity_buffer`", "\n", " ",
            "\n", "<b><i>", "6", "</i></b>. <b>", "\\[\\]", "</b> ",
            "SPL Token program - classic one", "\n", " ", "\n", "<b><i>", "7",
            "</i></b>. <b>", "\\[\\]", "</b> ",
            "SPL Token program to manage liquidity tokens. Either classic or 2022", "\n",
            " ", "\n", "<b><i>", "8", "</i></b>. <b>", "\\[\\]", "</b> ",
            "System Program", "\n", " ", "\n", "<b><i>", "9", " .. ", "9", " + ",
            "`reserves_count`", "</i></b>. <b>", "\\[\\]", "</b> ", "`reserves`", "\n",
            " ", "\n", "<b><i>", "9", " + ", "`reserves_count`", " .. ", "9", " + ",
            "`reserves_count`", " + ", "`lp_token_mints_count`", "</i></b>. <b>",
            "\\[\\]", "</b> ", "`lp_token_mints`", "\n", " ", "\n", "<b><i>", "9", " + ",
            "`reserves_count`", " + ", "`lp_token_mints_count`", " .. ", "9", " + ",
            "`reserves_count`", " + ", "`lp_token_mints_count`", " + ",
            "`lp_token_storages_count`", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "`lp_token_storages`", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [CreateVault]", " ",
            "(method [into_instruction][CreateVault::into_instruction]).", " ", "\n\n",
            " ",
            "For parse accounts infos from processor use struct [CreateVaultAccounts]",
            " ", "(method [from_iter][CreateVaultAccounts::from_iter]).", " ", "\n\n",
            " ", "For work with account indexes use struct [CreateVaultAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use create_vault;
    macro_rules! alter_vault {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Vault account to change.", "\n", " ", "\n",
            "<b><i>", "1", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "Authority assigned by Curator to manage Vaults.", "\n", " ", "\n", "<b><i>",
            "2", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "Authority who can change percentage in distribution.", "\n", " ", "\n",
            "<b><i>", "3", "</i></b>. <b>", "\\[\\]", "</b> ", "Curator of the Vault.",
            "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [AlterVault]", " ",
            "(method [into_instruction][AlterVault::into_instruction]).", " ", "\n\n",
            " ",
            "For parse accounts infos from processor use struct [AlterVaultAccounts]",
            " ", "(method [from_iter][AlterVaultAccounts::from_iter]).", " ", "\n\n",
            " ", "For work with account indexes use struct [AlterVaultAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use alter_vault;
    macro_rules! propose_distribution_item {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Vault account to change.", "\n", " ", "\n",
            "<b><i>", "1", "</i></b>. <b>", "\\[writable, signer\\]", "</b> ",
            "Authority assigned by Curator to manage Vaults.", "\n", " ", "\n", "<b><i>",
            "2", "</i></b>. <b>", "\\[\\]", "</b> ", "Curator of the Vault.", "\n", " ",
            "\n", "<b><i>", "3", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Reserve to be added in to distribution", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [ProposeDistributionItem]", " ",
            "(method [into_instruction][ProposeDistributionItem::into_instruction]).",
            " ", "\n\n", " ",
            "For parse accounts infos from processor use struct [ProposeDistributionItemAccounts]",
            " ", "(method [from_iter][ProposeDistributionItemAccounts::from_iter]).",
            " ", "\n\n", " ",
            "For work with account indexes use struct [ProposeDistributionItemAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use propose_distribution_item;
    macro_rules! add_distribution_item {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Vault account to change.", "\n", " ", "\n",
            "<b><i>", "1", "</i></b>. <b>", "\\[writable, signer\\]", "</b> ",
            "Authority assigned by Curator to manage Vaults.", "\n", " ", "\n", "<b><i>",
            "2", "</i></b>. <b>", "\\[\\]", "</b> ", "Curator of the Vault.", "\n", " ",
            "\n", "<b><i>", "3", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Mint of the reserve's LP tokens", "\n", " ", "\n", "<b><i>", "4",
            "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "LP token storage to be used by Vault. Unitialized.", "\n", " ", "\n",
            "<b><i>", "5", "</i></b>. <b>", "\\[\\]", "</b> ", "`program_authority`",
            "\n", " ", "\n", "<b><i>", "6", "</i></b>. <b>", "\\[\\]", "</b> ",
            "SPL Token Program", "\n", " ", "\n", "<b><i>", "7", "</i></b>. <b>",
            "\\[\\]", "</b> ", "System Program", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [AddDistributionItem]", " ",
            "(method [into_instruction][AddDistributionItem::into_instruction]).", " ",
            "\n\n", " ",
            "For parse accounts infos from processor use struct [AddDistributionItemAccounts]",
            " ", "(method [from_iter][AddDistributionItemAccounts::from_iter]).", " ",
            "\n\n", " ",
            "For work with account indexes use struct [AddDistributionItemAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use add_distribution_item;
    macro_rules! remove_distribution_item {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Vault account to change.", "\n", " ", "\n",
            "<b><i>", "1", "</i></b>. <b>", "\\[writable, signer\\]", "</b> ",
            "Authority assigned by Curator to manage Vaults.", "\n", " ", "\n", "<b><i>",
            "2", "</i></b>. <b>", "\\[\\]", "</b> ", "Curator of the Vault.", "\n", " ",
            "\n", "<b><i>", "3", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "LP token storage.", "\n", " ", "\n", "<b><i>", "4", "</i></b>. <b>",
            "\\[\\]", "</b> ", "`program_authority`", "\n", " ", "\n", "<b><i>", "5",
            "</i></b>. <b>", "\\[\\]", "</b> ", "SPL Token Program", "\n", "\n",
            " ## Usage", "\n", " ",
            "For create instruction use builder struct [RemoveDistributionItem]", " ",
            "(method [into_instruction][RemoveDistributionItem::into_instruction]).",
            " ", "\n\n", " ",
            "For parse accounts infos from processor use struct [RemoveDistributionItemAccounts]",
            " ", "(method [from_iter][RemoveDistributionItemAccounts::from_iter]).", " ",
            "\n\n", " ",
            "For work with account indexes use struct [RemoveDistributionItemAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use remove_distribution_item;
    macro_rules! create_position {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable, signer\\]", "</b> ", "Position account to create.", "\n", " ",
            "\n", "<b><i>", "1", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Vault account the position belongs to.", "\n", " ", "\n", "<b><i>", "2",
            "</i></b>. <b>", "\\[writable, signer\\]", "</b> ", "Position owner.", "\n",
            " ", "\n", "<b><i>", "3", "</i></b>. <b>", "\\[\\]", "</b> ",
            "`program_authority`", "\n", " ", "\n", "<b><i>", "4", "</i></b>. <b>",
            "\\[\\]", "</b> ", "System Program", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [CreatePosition]", " ",
            "(method [into_instruction][CreatePosition::into_instruction]).", " ",
            "\n\n", " ",
            "For parse accounts infos from processor use struct [CreatePositionAccounts]",
            " ", "(method [from_iter][CreatePositionAccounts::from_iter]).", " ", "\n\n",
            " ",
            "For work with account indexes use struct [CreatePositionAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use create_position;
    macro_rules! deposit {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Vault account to deposit to.", "\n", " ", "\n",
            "<b><i>", "1", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "Position account to deposit thru.", "\n", " ", "\n", "<b><i>", "2",
            "</i></b>. <b>", "\\[writable, signer\\]", "</b> ", "Position owner.", "\n",
            " ", "\n", "<b><i>", "3", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Liquidity tokens mint", "\n", " ", "\n", "<b><i>", "4", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "`liquidity_source`", "\n", " ", "\n", "<b><i>",
            "5", "</i></b>. <b>", "\\[writable\\]", "</b> ", "`liquidity_buffer`", "\n",
            " ", "\n", "<b><i>", "6", "</i></b>. <b>", "\\[\\]", "</b> ",
            "`program_authority`", "\n", " ", "\n", "<b><i>", "7", "</i></b>. <b>",
            "\\[\\]", "</b> ", "`superlendy_program_authority`", "\n", " ", "\n",
            "<b><i>", "8", "</i></b>. <b>", "\\[\\]", "</b> ",
            "SPL Token program - classic one", "\n", " ", "\n", "<b><i>", "9",
            "</i></b>. <b>", "\\[\\]", "</b> ",
            "SPL Token program to manage liquidity tokens. Either classic or 2022", "\n",
            " ", "\n", "<b><i>", "10", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Superlendy Program", "\n", " ", "\n", "<b><i>", "11", " .. ", "11", " + ",
            "`reserves_count`", "</i></b>. <b>", "\\[writable\\]", "</b> ", "`reserves`",
            "\n", " ", "\n", "<b><i>", "11", " + ", "`reserves_count`", " .. ", "11",
            " + ", "`reserves_count`", " + ", "`reserve_liquidity_supplies_count`",
            "</i></b>. <b>", "\\[writable\\]", "</b> ", "`reserve_liquidity_supplies`",
            "\n", " ", "\n", "<b><i>", "11", " + ", "`reserves_count`", " + ",
            "`reserve_liquidity_supplies_count`", " .. ", "11", " + ",
            "`reserves_count`", " + ", "`reserve_liquidity_supplies_count`", " + ",
            "`lp_token_mints_count`", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "`lp_token_mints`", "\n", " ", "\n", "<b><i>", "11", " + ",
            "`reserves_count`", " + ", "`reserve_liquidity_supplies_count`", " + ",
            "`lp_token_mints_count`", " .. ", "11", " + ", "`reserves_count`", " + ",
            "`reserve_liquidity_supplies_count`", " + ", "`lp_token_mints_count`", " + ",
            "`lp_token_storages_count`", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "`lp_token_storages`", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [Deposit]", " ",
            "(method [into_instruction][Deposit::into_instruction]).", " ", "\n\n", " ",
            "For parse accounts infos from processor use struct [DepositAccounts]", " ",
            "(method [from_iter][DepositAccounts::from_iter]).", " ", "\n\n", " ",
            "For work with account indexes use struct [DepositAccountIndexes].", "\n", }
        };
    }
    pub(crate) use deposit;
    macro_rules! withdraw {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Vault account to withdraw from.", "\n", " ",
            "\n", "<b><i>", "1", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "Position account to withdraw from.", "\n", " ", "\n", "<b><i>", "2",
            "</i></b>. <b>", "\\[writable, signer\\]", "</b> ", "Position owner.", "\n",
            " ", "\n", "<b><i>", "3", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Liquidity tokens mint", "\n", " ", "\n", "<b><i>", "4", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "`liquidity_destination`", "\n", " ", "\n",
            "<b><i>", "5", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "`liquidity_buffer`", "\n", " ", "\n", "<b><i>", "6", "</i></b>. <b>",
            "\\[\\]", "</b> ", "`program_authority`", "\n", " ", "\n", "<b><i>", "7",
            "</i></b>. <b>", "\\[\\]", "</b> ", "`superlendy_program_authority`", "\n",
            " ", "\n", "<b><i>", "8", "</i></b>. <b>", "\\[\\]", "</b> ",
            "SPL Token program - classic one", "\n", " ", "\n", "<b><i>", "9",
            "</i></b>. <b>", "\\[\\]", "</b> ",
            "SPL Token program to manage liquidity tokens. Either classic or 2022", "\n",
            " ", "\n", "<b><i>", "10", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Superlendy Program", "\n", " ", "\n", "<b><i>", "11", " .. ", "11", " + ",
            "`reserves_count`", "</i></b>. <b>", "\\[writable\\]", "</b> ", "`reserves`",
            "\n", " ", "\n", "<b><i>", "11", " + ", "`reserves_count`", " .. ", "11",
            " + ", "`reserves_count`", " + ", "`reserve_liquidity_supplies_count`",
            "</i></b>. <b>", "\\[writable\\]", "</b> ", "`reserve_liquidity_supplies`",
            "\n", " ", "\n", "<b><i>", "11", " + ", "`reserves_count`", " + ",
            "`reserve_liquidity_supplies_count`", " .. ", "11", " + ",
            "`reserves_count`", " + ", "`reserve_liquidity_supplies_count`", " + ",
            "`lp_token_mints_count`", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "`lp_token_mints`", "\n", " ", "\n", "<b><i>", "11", " + ",
            "`reserves_count`", " + ", "`reserve_liquidity_supplies_count`", " + ",
            "`lp_token_mints_count`", " .. ", "11", " + ", "`reserves_count`", " + ",
            "`reserve_liquidity_supplies_count`", " + ", "`lp_token_mints_count`", " + ",
            "`lp_token_storages_count`", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "`lp_token_storages`", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [Withdraw]", " ",
            "(method [into_instruction][Withdraw::into_instruction]).", " ", "\n\n", " ",
            "For parse accounts infos from processor use struct [WithdrawAccounts]", " ",
            "(method [from_iter][WithdrawAccounts::from_iter]).", " ", "\n\n", " ",
            "For work with account indexes use struct [WithdrawAccountIndexes].", "\n", }
        };
    }
    pub(crate) use withdraw;
    macro_rules! close_position {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Position account to close.", "\n", " ", "\n",
            "<b><i>", "1", "</i></b>. <b>", "\\[\\]", "</b> ", "`vault`", "\n", " ",
            "\n", "<b><i>", "2", "</i></b>. <b>", "\\[writable, signer\\]", "</b> ",
            "Owner of the position", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [ClosePosition]", " ",
            "(method [into_instruction][ClosePosition::into_instruction]).", " ", "\n\n",
            " ",
            "For parse accounts infos from processor use struct [ClosePositionAccounts]",
            " ", "(method [from_iter][ClosePositionAccounts::from_iter]).", " ", "\n\n",
            " ",
            "For work with account indexes use struct [ClosePositionAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use close_position;
    macro_rules! rebalance {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[\\]", "</b> ", "`vault`", "\n", " ", "\n", "<b><i>", "1",
            "</i></b>. <b>", "\\[\\]", "</b> ", "`program_authority`", "\n", " ", "\n",
            "<b><i>", "2", "</i></b>. <b>", "\\[\\]", "</b> ", "Liquidity tokens mint",
            "\n", " ", "\n", "<b><i>", "3", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "`liquidity_buffer`", "\n", " ", "\n", "<b><i>", "4", "</i></b>. <b>",
            "\\[\\]", "</b> ", "`superlendy_program_authority`", "\n", " ", "\n",
            "<b><i>", "5", "</i></b>. <b>", "\\[\\]", "</b> ",
            "SPL Token program - classic one", "\n", " ", "\n", "<b><i>", "6",
            "</i></b>. <b>", "\\[\\]", "</b> ",
            "SPL Token program to manage liquidity tokens. Either classic or 2022", "\n",
            " ", "\n", "<b><i>", "7", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Superlendy Program", "\n", " ", "\n", "<b><i>", "8", " .. ", "8", " + ",
            "`reserves_count`", "</i></b>. <b>", "\\[writable\\]", "</b> ", "`reserves`",
            "\n", " ", "\n", "<b><i>", "8", " + ", "`reserves_count`", " .. ", "8",
            " + ", "`reserves_count`", " + ", "`reserve_liquidity_supplies_count`",
            "</i></b>. <b>", "\\[writable\\]", "</b> ", "`reserve_liquidity_supplies`",
            "\n", " ", "\n", "<b><i>", "8", " + ", "`reserves_count`", " + ",
            "`reserve_liquidity_supplies_count`", " .. ", "8", " + ", "`reserves_count`",
            " + ", "`reserve_liquidity_supplies_count`", " + ", "`lp_token_mints_count`",
            "</i></b>. <b>", "\\[writable\\]", "</b> ", "`lp_token_mints`", "\n", " ",
            "\n", "<b><i>", "8", " + ", "`reserves_count`", " + ",
            "`reserve_liquidity_supplies_count`", " + ", "`lp_token_mints_count`",
            " .. ", "8", " + ", "`reserves_count`", " + ",
            "`reserve_liquidity_supplies_count`", " + ", "`lp_token_mints_count`", " + ",
            "`lp_token_storages_count`", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "`lp_token_storages`", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [Rebalance]", " ",
            "(method [into_instruction][Rebalance::into_instruction]).", " ", "\n\n",
            " ",
            "For parse accounts infos from processor use struct [RebalanceAccounts]",
            " ", "(method [from_iter][RebalanceAccounts::from_iter]).", " ", "\n\n", " ",
            "For work with account indexes use struct [RebalanceAccountIndexes].", "\n",
            }
        };
    }
    pub(crate) use rebalance;
    macro_rules! init_reward_supply {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ",
            "Reward supply account to initialize. Uninitialized. PDA", "\n", " ", "\n",
            "<b><i>", "1", "</i></b>. <b>", "\\[\\]", "</b> ", "Reward token mint.",
            "\n", " ", "\n", "<b><i>", "2", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Vault to init reward supply for.", "\n", " ", "\n", "<b><i>", "3",
            "</i></b>. <b>", "\\[writable, signer\\]", "</b> ",
            "Authority who can manage Vault.", "\n", " ", "\n", "<b><i>", "4",
            "</i></b>. <b>", "\\[\\]", "</b> ", "Curator account.", "\n", " ", "\n",
            "<b><i>", "5", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Contract's reward authority. PDA.", "\n", " ", "\n", "<b><i>", "6",
            "</i></b>. <b>", "\\[\\]", "</b> ", "SPL Token program", "\n", " ", "\n",
            "<b><i>", "7", "</i></b>. <b>", "\\[\\]", "</b> ", "System Program.", "\n",
            "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [InitRewardSupply]", " ",
            "(method [into_instruction][InitRewardSupply::into_instruction]).", " ",
            "\n\n", " ",
            "For parse accounts infos from processor use struct [InitRewardSupplyAccounts]",
            " ", "(method [from_iter][InitRewardSupplyAccounts::from_iter]).", " ",
            "\n\n", " ",
            "For work with account indexes use struct [InitRewardSupplyAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use init_reward_supply;
    macro_rules! set_reward_rules {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Vault account to set reward rules for", "\n",
            " ", "\n", "<b><i>", "1", "</i></b>. <b>", "\\[writable, signer\\]", "</b> ",
            "Authority who can configure Vault.", "\n", " ", "\n", "<b><i>", "2",
            "</i></b>. <b>", "\\[\\]", "</b> ", "Curator account.", "\n", "\n",
            " ## Usage", "\n", " ",
            "For create instruction use builder struct [SetRewardRules]", " ",
            "(method [into_instruction][SetRewardRules::into_instruction]).", " ",
            "\n\n", " ",
            "For parse accounts infos from processor use struct [SetRewardRulesAccounts]",
            " ", "(method [from_iter][SetRewardRulesAccounts::from_iter]).", " ", "\n\n",
            " ",
            "For work with account indexes use struct [SetRewardRulesAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use set_reward_rules;
    macro_rules! claim_reward {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ",
            "Position account to claim rewords for. Refreshed.", "\n", " ", "\n",
            "<b><i>", "1", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "Contract managed SPL token wallet which holds reward tokens. PDA.", "\n",
            " ", "\n", "<b><i>", "2", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "User's SPL token wallet which will receive reward tokens", "\n", " ", "\n",
            "<b><i>", "3", "</i></b>. <b>", "\\[signer\\]", "</b> ", "Position owner",
            "\n", " ", "\n", "<b><i>", "4", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Vault, position belongs to.", "\n", " ", "\n", "<b><i>", "5",
            "</i></b>. <b>", "\\[\\]", "</b> ",
            "Reward token mint to claim. Determines which reward will be claimed from the Position.",
            "\n", " ", "\n", "<b><i>", "6", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Contract's reward authority. PDA.", "\n", " ", "\n", "<b><i>", "7",
            "</i></b>. <b>", "\\[\\]", "</b> ", "SPL Token program", "\n", "\n",
            " ## Usage", "\n", " ",
            "For create instruction use builder struct [ClaimReward]", " ",
            "(method [into_instruction][ClaimReward::into_instruction]).", " ", "\n\n",
            " ",
            "For parse accounts infos from processor use struct [ClaimRewardAccounts]",
            " ", "(method [from_iter][ClaimRewardAccounts::from_iter]).", " ", "\n\n",
            " ", "For work with account indexes use struct [ClaimRewardAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use claim_reward;
    macro_rules! withdraw_reward {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ",
            "Contract managed SPL token wallet which holds reward tokens. PDA.", "\n",
            " ", "\n", "<b><i>", "1", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "SPL token account of reward_mint to receive reward tokens.", "\n", " ",
            "\n", "<b><i>", "2", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Vault to withdraw rewards from.", "\n", " ", "\n", "<b><i>", "3",
            "</i></b>. <b>", "\\[writable, signer\\]", "</b> ",
            "Authority who can configure Vault.", "\n", " ", "\n", "<b><i>", "4",
            "</i></b>. <b>", "\\[\\]", "</b> ", "Curator account.", "\n", " ", "\n",
            "<b><i>", "5", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Reward token mint to withdraw.", "\n", " ", "\n", "<b><i>", "6",
            "</i></b>. <b>", "\\[\\]", "</b> ", "Contract's reward authority. PDA.",
            "\n", " ", "\n", "<b><i>", "7", "</i></b>. <b>", "\\[\\]", "</b> ",
            "SPL Token program", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [WithdrawReward]", " ",
            "(method [into_instruction][WithdrawReward::into_instruction]).", " ",
            "\n\n", " ",
            "For parse accounts infos from processor use struct [WithdrawRewardAccounts]",
            " ", "(method [from_iter][WithdrawRewardAccounts::from_iter]).", " ", "\n\n",
            " ",
            "For work with account indexes use struct [WithdrawRewardAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use withdraw_reward;
    macro_rules! claim_fees {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Vault account to claim fees from.", "\n", " ",
            "\n", "<b><i>", "1", "</i></b>. <b>", "\\[signer\\]", "</b> ",
            "Authority assigned by Curator to manage Vaults.", "\n", " ", "\n", "<b><i>",
            "2", "</i></b>. <b>", "\\[\\]", "</b> ", "Curator of the Vault.", "\n", " ",
            "\n", "<b><i>", "3", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Liquidity tokens mint", "\n", " ", "\n", "<b><i>", "4", "</i></b>. <b>",
            "\\[writable\\]", "</b> ",
            "Any SPL account of proper mint to transfer fees to", "\n", " ", "\n",
            "<b><i>", "5", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "Any SPL account of proper mint to transfer fees to", "\n", " ", "\n",
            "<b><i>", "6", "</i></b>. <b>", "\\[\\]", "</b> ", "`program_authority`",
            "\n", " ", "\n", "<b><i>", "7", "</i></b>. <b>", "\\[\\]", "</b> ",
            "`superlendy_program_authority`", "\n", " ", "\n", "<b><i>", "8",
            "</i></b>. <b>", "\\[\\]", "</b> ", "SPL Token program - classic one", "\n",
            " ", "\n", "<b><i>", "9", "</i></b>. <b>", "\\[\\]", "</b> ",
            "SPL Token program to manage liquidity tokens. Either classic or 2022", "\n",
            " ", "\n", "<b><i>", "10", "</i></b>. <b>", "\\[\\]", "</b> ",
            "Superlendy Program", "\n", " ", "\n", "<b><i>", "11", " .. ", "11", " + ",
            "`reserves_count`", "</i></b>. <b>", "\\[writable\\]", "</b> ", "`reserves`",
            "\n", " ", "\n", "<b><i>", "11", " + ", "`reserves_count`", " .. ", "11",
            " + ", "`reserves_count`", " + ", "`reserve_liquidity_supplies_count`",
            "</i></b>. <b>", "\\[writable\\]", "</b> ", "`reserve_liquidity_supplies`",
            "\n", " ", "\n", "<b><i>", "11", " + ", "`reserves_count`", " + ",
            "`reserve_liquidity_supplies_count`", " .. ", "11", " + ",
            "`reserves_count`", " + ", "`reserve_liquidity_supplies_count`", " + ",
            "`lp_token_mints_count`", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "`lp_token_mints`", "\n", " ", "\n", "<b><i>", "11", " + ",
            "`reserves_count`", " + ", "`reserve_liquidity_supplies_count`", " + ",
            "`lp_token_mints_count`", " .. ", "11", " + ", "`reserves_count`", " + ",
            "`reserve_liquidity_supplies_count`", " + ", "`lp_token_mints_count`", " + ",
            "`lp_token_storages_count`", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "`lp_token_storages`", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [ClaimFees]", " ",
            "(method [into_instruction][ClaimFees::into_instruction]).", " ", "\n\n",
            " ",
            "For parse accounts infos from processor use struct [ClaimFeesAccounts]",
            " ", "(method [from_iter][ClaimFeesAccounts::from_iter]).", " ", "\n\n", " ",
            "For work with account indexes use struct [ClaimFeesAccountIndexes].", "\n",
            }
        };
    }
    pub(crate) use claim_fees;
}
