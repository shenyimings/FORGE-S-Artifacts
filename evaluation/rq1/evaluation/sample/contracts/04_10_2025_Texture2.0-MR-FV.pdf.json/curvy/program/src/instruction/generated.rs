#![allow(unexpected_cfgs)]
use super::*;
///[CurvyInstruction::CreateCurve] Builder struct
pub struct CreateCurve {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Curve account to create.
    pub curve: solana_program::pubkey::Pubkey,
    ///Curve owner.
    pub owner: solana_program::pubkey::Pubkey,
    pub params: CurveParams,
}
impl CreateCurve {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            curve,
            owner,
            params,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = crate::ID;
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts.extend([solana_program::instruction::AccountMeta::new(curve, true)]);
        accounts.extend([solana_program::instruction::AccountMeta::new(owner, true)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    solana_program::system_program::ID,
                    false,
                ),
            ]);
        let ix = CurvyInstruction::CreateCurve {
            params,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[CurvyInstruction::AlterCurve] Builder struct
pub struct AlterCurve {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Curve account to update.
    pub curve: solana_program::pubkey::Pubkey,
    ///Curve owner.
    pub owner: solana_program::pubkey::Pubkey,
    pub params: CurveParams,
}
impl AlterCurve {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            curve,
            owner,
            params,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = crate::ID;
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts.extend([solana_program::instruction::AccountMeta::new(curve, false)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(owner, true),
            ]);
        let ix = CurvyInstruction::AlterCurve {
            params,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[CurvyInstruction::DeleteCurve] Builder struct
pub struct DeleteCurve {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Curve account to delete.
    pub curve: solana_program::pubkey::Pubkey,
    ///Curve owner.
    pub owner: solana_program::pubkey::Pubkey,
}
impl DeleteCurve {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self { #[cfg(feature = "program-id-manually")] program_id, curve, owner } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = crate::ID;
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts.extend([solana_program::instruction::AccountMeta::new(curve, false)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(owner, true),
            ]);
        let ix = CurvyInstruction::DeleteCurve {};
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
/// [CurvyInstruction::CreateCurve] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct CreateCurveAccountIndexes {
    pub curve: usize,
    pub owner: usize,
    pub system_program: usize,
}
impl CreateCurveAccountIndexes {
    pub const COUNT: usize = 3usize;
    pub const CURVE: usize = 0usize;
    pub const OWNER: usize = 1usize;
    pub const SYSTEM_PROGRAM: usize = 2usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            curve: iter.next().unwrap(),
            owner: iter.next().unwrap(),
            system_program: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            curve: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            owner: {
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
impl<'a> TryFrom<&'a [u8]> for CreateCurveAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for CreateCurveAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for CreateCurveAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for CreateCurveAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [CurvyInstruction::AlterCurve] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct AlterCurveAccountIndexes {
    pub curve: usize,
    pub owner: usize,
}
impl AlterCurveAccountIndexes {
    pub const COUNT: usize = 2usize;
    pub const CURVE: usize = 0usize;
    pub const OWNER: usize = 1usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            curve: iter.next().unwrap(),
            owner: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            curve: {
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
impl<'a> TryFrom<&'a [u8]> for AlterCurveAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for AlterCurveAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for AlterCurveAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for AlterCurveAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [CurvyInstruction::DeleteCurve] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct DeleteCurveAccountIndexes {
    pub curve: usize,
    pub owner: usize,
}
impl DeleteCurveAccountIndexes {
    pub const COUNT: usize = 2usize;
    pub const CURVE: usize = 0usize;
    pub const OWNER: usize = 1usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            curve: iter.next().unwrap(),
            owner: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            curve: {
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
impl<'a> TryFrom<&'a [u8]> for DeleteCurveAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for DeleteCurveAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for DeleteCurveAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for DeleteCurveAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
///[CurvyInstruction::CreateCurve] instruction account infos helper
#[derive(Debug)]
pub struct CreateCurveAccounts<'a, 'i> {
    ///Curve account to create.
    pub curve: &'a solana_program::account_info::AccountInfo<'i>,
    ///Curve owner.
    pub owner: &'a solana_program::account_info::AccountInfo<'i>,
    ///System program
    pub system_program: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> CreateCurveAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let curve = texture_common::utils::next_account_info(iter)?;
        let owner = texture_common::utils::next_account_info(iter)?;
        let system_program = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &crate::ID,
            "self_program_id",
        )?;
        if !curve.is_writable {
            solana_program::msg!(concat!(stringify!(curve), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*curve.key).into());
        }
        if !curve.is_signer {
            return Err(texture_common::error::MissingSignature(*curve.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            curve.owner,
            &solana_program::system_program::ID,
            concat!(stringify!(curve), " owner"),
        )?;
        if curve.data_len() != 0 {
            solana_program::msg!(
                concat!("invalid ", stringify!(curve), " account size")
            );
            return Err(texture_common::error::InvalidAccount(*curve.key).into());
        }
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
            curve,
            owner,
            system_program,
        })
    }
}
///[CurvyInstruction::AlterCurve] instruction account infos helper
#[derive(Debug)]
pub struct AlterCurveAccounts<'a, 'i> {
    ///Curve account to update.
    pub curve: &'a solana_program::account_info::AccountInfo<'i>,
    ///Curve owner.
    pub owner: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> AlterCurveAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let curve = texture_common::utils::next_account_info(iter)?;
        let owner = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &crate::ID,
            "self_program_id",
        )?;
        if !curve.is_writable {
            solana_program::msg!(concat!(stringify!(curve), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*curve.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            curve.owner,
            &__self_program_id__,
            concat!(stringify!(curve), " owner"),
        )?;
        if !owner.is_signer {
            return Err(texture_common::error::MissingSignature(*owner.key).into());
        }
        Ok(Self { curve, owner })
    }
}
///[CurvyInstruction::DeleteCurve] instruction account infos helper
#[derive(Debug)]
pub struct DeleteCurveAccounts<'a, 'i> {
    ///Curve account to delete.
    pub curve: &'a solana_program::account_info::AccountInfo<'i>,
    ///Curve owner.
    pub owner: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> DeleteCurveAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let curve = texture_common::utils::next_account_info(iter)?;
        let owner = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &crate::ID,
            "self_program_id",
        )?;
        if !curve.is_writable {
            solana_program::msg!(concat!(stringify!(curve), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*curve.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            curve.owner,
            &__self_program_id__,
            concat!(stringify!(curve), " owner"),
        )?;
        if !owner.is_signer {
            return Err(texture_common::error::MissingSignature(*owner.key).into());
        }
        Ok(Self { curve, owner })
    }
}
pub(crate) mod ix_docs {
    macro_rules! create_curve {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable, signer\\]", "</b> ", "Curve account to create.", "\n", " ",
            "\n", "<b><i>", "1", "</i></b>. <b>", "\\[writable, signer\\]", "</b> ",
            "Curve owner.", "\n", " ", "\n", "<b><i>", "2", "</i></b>. <b>", "\\[\\]",
            "</b> ", "System program", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [CreateCurve]", " ",
            "(method [into_instruction][CreateCurve::into_instruction]).", " ", "\n\n",
            " ",
            "For parse accounts infos from processor use struct [CreateCurveAccounts]",
            " ", "(method [from_iter][CreateCurveAccounts::from_iter]).", " ", "\n\n",
            " ", "For work with account indexes use struct [CreateCurveAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use create_curve;
    macro_rules! alter_curve {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Curve account to update.", "\n", " ", "\n",
            "<b><i>", "1", "</i></b>. <b>", "\\[signer\\]", "</b> ", "Curve owner.",
            "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [AlterCurve]", " ",
            "(method [into_instruction][AlterCurve::into_instruction]).", " ", "\n\n",
            " ",
            "For parse accounts infos from processor use struct [AlterCurveAccounts]",
            " ", "(method [from_iter][AlterCurveAccounts::from_iter]).", " ", "\n\n",
            " ", "For work with account indexes use struct [AlterCurveAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use alter_curve;
    macro_rules! delete_curve {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Curve account to delete.", "\n", " ", "\n",
            "<b><i>", "1", "</i></b>. <b>", "\\[signer\\]", "</b> ", "Curve owner.",
            "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [DeleteCurve]", " ",
            "(method [into_instruction][DeleteCurve::into_instruction]).", " ", "\n\n",
            " ",
            "For parse accounts infos from processor use struct [DeleteCurveAccounts]",
            " ", "(method [from_iter][DeleteCurveAccounts::from_iter]).", " ", "\n\n",
            " ", "For work with account indexes use struct [DeleteCurveAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use delete_curve;
}
