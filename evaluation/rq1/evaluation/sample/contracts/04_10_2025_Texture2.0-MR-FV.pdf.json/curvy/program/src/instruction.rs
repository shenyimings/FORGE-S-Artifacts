use borsh::{BorshDeserialize, BorshSerialize};
use texture_common::macros::Instruction;

use crate::state::curve::CurveParams;

#[derive(Instruction, BorshSerialize, BorshDeserialize, Debug)]
#[instruction(
    out_dir = "src/instruction",
    out_mod = "generated",
    program_id = crate::ID,
    docs_module = ix_docs,
)]
pub enum CurvyInstruction {
    /// Create Curve account
    ///
    #[doc = ix_docs::create_curve!()]
    #[accounts(
        account(
            name = "curve",
            flags(writable, signer),
            docs = ["Curve account to create."],
            checks(owner = "system", size = 0),
        ),
        account(
            name = "owner",
            flags(writable, signer),
            docs = ["Curve owner."],
        ),
        program(id = "system", docs = ["System program"])
    )]
    CreateCurve { params: CurveParams },
    /// Alter existing Curve
    ///
    #[doc = ix_docs::alter_curve!()]
    #[accounts(
        account(
            name = "curve",
            flags(writable),
            docs = ["Curve account to update."],
            checks(owner = "self"),
        ),
        account(
            name = "owner",
            flags(signer),
            docs = ["Curve owner."],
        ),
    )]
    AlterCurve { params: CurveParams },
    /// Delete existing Curve
    ///
    #[doc = ix_docs::delete_curve!()]
    #[accounts(
        account(
            name = "curve",
            flags(writable),
            docs = ["Curve account to delete."],
            checks(owner = "self"),
        ),
        account(
            name = "owner",
            flags(signer),
            docs = ["Curve owner."],
        ),
    )]
    DeleteCurve,
}
