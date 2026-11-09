use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use solana_program::msg;
use solana_program::pubkey::Pubkey;

use crate::error::CurvyError;
use crate::CurvyResult;
use texture_common::account::{PodAccount, PodAccountError};
use texture_common::math::{CheckedAdd, CheckedMul, Decimal};

use crate::state::CURVE_DISCRIMINATOR;

pub const SYMBOL_MAX_SIZE: usize = 16;

static_assertions::const_assert_eq!(Curve::SIZE, std::mem::size_of::<Curve>());
static_assertions::const_assert_eq!(0, std::mem::size_of::<Curve>() % 8);

/// These are fixed point decimal number with precision specified in Curve.
/// X holds utilization rate in percents e.g. 45.2344 % thus usually we don't need values more than 100.
/// But we need decimal values in that range. Thus u32 with 6 decimals will allow to express 4294.967295
/// maximum value. Which is enough.
/// Y holds APR which can be more than 100% but its hardly possible that it will be more than 4294.967295.
/// Thus u32 with 6 decimals will also work.
pub type CurveX = u32;
pub type CurveY = u32;

/// To make design simple we limit number of `y` samples. This allows send all Curve data
/// in one TX and to allocate statically known space in the account.
pub const MAX_Y_CNT: usize = 130;

#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone)]
pub struct CurveParams {
    pub name: [u8; SYMBOL_MAX_SIZE],
    pub formula: [u8; SYMBOL_MAX_SIZE],
    /// Starting X coordinate
    pub x0: CurveX,
    /// Step on X scale between Y samples
    pub x_step: CurveX,
    /// Number of samples in `y` array
    pub y_count: u8,
    /// Precision of
    pub decimals: u8,
    /// Array of `y` values
    pub y: [CurveY; MAX_Y_CNT],
}

impl CurveParams {
    pub fn new(
        name: &str,
        formula: &str,
        x0: CurveX,
        x_step: u32,
        y_count: u8,
        decimals: u8,
        y: [CurveY; MAX_Y_CNT],
    ) -> Self {
        Self {
            name: super::utils::str_to_array(name),
            formula: super::utils::str_to_array(formula),
            x0,
            x_step,
            y_count,
            decimals,
            y,
        }
    }
}

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Curve {
    pub discriminator: [u8; 8],
    pub version: u8,

    pub _padding: [u8; 7],

    /// a human-readable name
    pub name: [u8; SYMBOL_MAX_SIZE],

    /// a human-readable formula
    pub formula: [u8; SYMBOL_MAX_SIZE],

    /// authority who has full rights to manage that account
    pub owner: Pubkey,

    /// Starting X coordinate (4 bytes)
    pub x0: CurveX,

    /// Step on X scale between Y samples (4 bytes)
    pub x_step: CurveX,

    /// Number of samples in `y` array (2 bytes)
    pub y_count: u8,

    /// Decimals number for x0, x_step, y.
    pub decimals: u8,

    pub _padding1: [u8; 6],

    /// Array of `y` values
    pub y: [CurveY; MAX_Y_CNT],
}

impl Curve {
    pub fn set_params(&mut self, params: CurveParams) {
        let Self {
            discriminator,
            version,
            _padding,
            name,
            formula,
            x0,
            x_step,
            y_count,
            owner: _,
            decimals,
            _padding1,
            y,
        } = self;

        *discriminator = *CURVE_DISCRIMINATOR;
        *version = Self::VERSION;
        *_padding = Zeroable::zeroed();
        *name = params.name;
        *formula = params.formula;
        *x0 = params.x0;
        *x_step = params.x_step;
        *y_count = params.y_count;
        *decimals = params.decimals;
        *_padding1 = Zeroable::zeroed();
        *y = params.y;
    }

    /// Checks that x0, x_step, y_count are aligned with each other
    pub fn check_params(params: &CurveParams) -> CurvyResult<()> {
        if params.x_step == 0 {
            msg!("x_step must be non zero");
            return Err(CurvyError::InvalidParams);
        }

        if params.y_count == 0 {
            msg!("y_count must be non zero");
            return Err(CurvyError::InvalidParams);
        }

        if params.decimals > 9 {
            msg!("decimals must be in range [0, 9]");
            return Err(CurvyError::InvalidParams);
        }

        // maximum X coordinate value should not be bigger then maximum value CurveX can hold with
        // given decimals
        let max_x = Decimal::from_i128_with_scale(params.x0 as i128, params.decimals as u32)?
            .checked_add(
                Decimal::from_i128_with_scale(params.x_step as i128, params.decimals as u32)?
                    .checked_mul(Decimal::from_i128_with_scale(params.y_count as i128, 0)?)?,
            )?;
        let u32_max = Decimal::from_i128_with_scale(u32::MAX as i128, params.decimals as u32)?;

        if max_x > u32_max {
            msg!("Provided x0, x_step and y_count results in too big maximum X value {}. It should not exceed {}", max_x, u32_max);
            return Err(CurvyError::InvalidParams);
        }

        if max_x <= Decimal::from_i128_with_scale(params.x0 as i128, 0)? {
            msg!("y_count*x_step results in very small number. Choose other value so x0 will be less then calculated max_x");
            return Err(CurvyError::InvalidParams);
        }

        Ok(())
    }
}

impl PodAccount for Curve {
    const DISCRIMINATOR: &'static [u8] = CURVE_DISCRIMINATOR;

    type Version = u8;

    const VERSION: Self::Version = 1;

    type InitParams = (/*params:*/ CurveParams, /*owner:*/ Pubkey);

    type InitError = PodAccountError;

    fn discriminator(&self) -> &[u8] {
        &self.discriminator
    }

    fn version(&self) -> Self::Version {
        self.version
    }

    fn init_unckecked(
        &mut self,
        (params, owner_key): Self::InitParams,
    ) -> Result<(), Self::InitError> {
        self.set_params(params);
        self.owner = owner_key;

        Ok(())
    }
}
