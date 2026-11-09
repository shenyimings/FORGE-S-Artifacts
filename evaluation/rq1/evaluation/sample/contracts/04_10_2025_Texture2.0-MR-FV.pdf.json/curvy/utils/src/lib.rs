use anyhow::Result;

use texture_common::account::PodAccount;
use texture_common::math::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Decimal};

use curvy::state::curve::Curve;

/// Calculates Y value in given X point using linear interpolation between X0 < X < X1 points.
/// Expects raw Curvy account data as input.
pub fn calc_y_raw(x: Decimal, curve_account_data: &[u8]) -> Result<Decimal, String> {
    let curve = Curve::try_from_bytes(curve_account_data)
        .map_err(|_err| String::from("error unpacking Curve account"))?;

    calc_y(x, curve).map_err(|err| err.to_string())
}

/// Calculates Y value in given X point using linear interpolation between X0 < X < X1 points.
/// Based on deserialized Curve account
/// `x` - is human-readable number WITHOUT any knowledge about decimals inside Curve.
pub fn calc_y(x1: Decimal, curve: &Curve) -> texture_common::math::MathResult<Decimal> {
    let &Curve {
        x0,
        x_step,
        y_count,
        decimals,
        y,
        ..
    } = curve;

    calc_y_with_params(
        &y[0..y_count as usize],
        decimals,
        x_step,
        Decimal::from_i128_with_scale(x0 as i128, 0)?,
        x1,
    )
}

pub fn calc_y_with_params(
    y: &[u32],
    decimals: u8,
    x_step: u32,
    x0: Decimal,
    x: Decimal,
) -> texture_common::math::MathResult<Decimal> {
    let x_last = {
        let rhs = || y.len().checked_sub(1)?.checked_mul(x_step as usize);
        x0.checked_add(Decimal::from_i128_with_scale(
            rhs().ok_or(texture_common::math::MathError(format!(
                "calc last x rhs failure: y_len={}, x_step={x_step}",
                y.len()
            )))? as i128,
            0,
        )?)?
    };

    // Adjust X to be on the same scale as x0 and x_step
    let x_scaled =
        x.checked_mul(Decimal::from_i128_with_scale(10, 0)?.checked_pow(decimals as u64)?)?;

    if !(x0..=x_last).contains(&x_scaled) {
        return Err(texture_common::math::MathError(format!(
            "x_scaled={x_scaled} is out of function range {x0}..={x_last}"
        )));
    }

    let x_idx_dec = {
        let x_step_dec = Decimal::from_i128_with_scale(x_step as i128, 0)?;
        x_scaled.checked_sub(x0)?.checked_div(x_step_dec)?
    };
    let pre_x_idx = x_idx_dec.floor()?;
    if x_idx_dec == Decimal::from_i128_with_scale(pre_x_idx as i128, 0)? {
        // current `x` is integer thus just get y from table
        //
        // NOTE: for prevent index out of array bounds
        // (when `x` is MAX, `post_x_idx = last_x_idx + 1`)
        return Decimal::from_i128_with_scale(
            *y.get(pre_x_idx as usize)
                .ok_or(texture_common::math::MathError(format!(
                    "get y failure: idx={pre_x_idx}"
                )))? as i128,
            decimals as u32,
        );
    }

    let post_x_idx = pre_x_idx
        .checked_add(1)
        .ok_or(texture_common::math::MathError(format!(
            "calc post x idx failure: pre idx={pre_x_idx}"
        )))?;

    let (pre_x, post_x) = {
        let rhs = |idx: u64| idx.checked_mul(x_step as u64);
        (
            x0.checked_add(Decimal::from_i128_with_scale(
                rhs(pre_x_idx).ok_or(texture_common::math::MathError(format!(
                    "calc pre x rhs failure: idx={pre_x_idx}, step={x_step}"
                )))? as i128,
                0,
            )?)?,
            x0.checked_add(Decimal::from_i128_with_scale(
                rhs(post_x_idx).ok_or(texture_common::math::MathError(format!(
                    "calc post x rhs failure: idx={post_x_idx}, step={x_step}"
                )))? as i128,
                0,
            )?)?,
        )
    };

    let pre_y = Decimal::from_i128_with_scale(
        *y.get(pre_x_idx as usize)
            .ok_or(texture_common::math::MathError(format!(
                "get pre y failure, idx={pre_x_idx}"
            )))? as i128,
        decimals as u32,
    )?;
    let post_y = Decimal::from_i128_with_scale(
        *y.get(post_x_idx as usize)
            .ok_or(texture_common::math::MathError(format!(
                "get post y failure, idx={post_x_idx}"
            )))? as i128,
        decimals as u32,
    )?;

    // count how much percentage x takes up on it's nearest segment
    let diff_x = post_x.checked_sub(pre_x)?;
    let n = x_scaled.checked_sub(pre_x)?.checked_div(diff_x)?;

    // multiply y's segment length to the percentage and count the result
    let diff_y = post_y.checked_sub(pre_y)?;
    let y = diff_y.checked_mul(n)?.checked_add(pre_y)?;

    Ok(y)
}

#[cfg(test)]
mod tests {
    use curvy::state::curve::{CurveParams, CurveY, MAX_Y_CNT};
    use curvy::state::utils;
    use texture_common::_export::Pubkey;

    use super::*;

    const Y: [CurveY; 5] = [200, 300, 400, 700, 1_000_000_000];

    #[test]
    fn calc() {
        let mut y = [0; MAX_Y_CNT];
        y[..Y.len()].copy_from_slice(&Y);

        let x_max = Decimal::from_i128_with_scale(8, 2).unwrap();

        // X range is 0 - 0.08. This is points 0; 0.02; 0.04; 0.06; 0.08;
        let params = CurveParams {
            name: utils::str_to_array("test curve"),
            formula: utils::str_to_array("y=f(x)"),
            x0: 0,
            x_step: 2,
            y_count: Y.len() as u8,
            decimals: 2,
            y,
        };

        let curve = Curve::from_init_params((params, Pubkey::default()));

        // check first value of function
        let x = Decimal::ZERO;
        let res = calc_y(x, &curve).unwrap();
        assert_eq!(
            res,
            Decimal::from_i128_with_scale(200, 2).unwrap(),
            "precounted first value is not matching with function result"
        );

        // check last value of function. X - is like human perceive it i.e. 0.08
        let res = calc_y(x_max, &curve).unwrap();
        assert_eq!(
            res,
            Decimal::from_i128_with_scale(1_000_000_000, 2).unwrap(),
            "precounted last value is not matching with function result"
        );

        // check bound before first. x = -0.01
        let x = Decimal::from_i128_with_scale(-1, 2).unwrap();
        let res = calc_y(x, &curve);
        assert!(res.is_err(), "out of bounds (before first)");

        // check bound after last. x = 0.11
        let x = Decimal::from_i128_with_scale(8 + 1, 2).unwrap();
        let res = calc_y(x, &curve);
        assert!(res.is_err(), "out of bounds (after last)");

        // Value in the middle of X0-X1 should give y = (200+300) / 2
        let x = Decimal::from_i128_with_scale(1, 2).unwrap();
        let res = calc_y(x, &curve).unwrap();
        assert_eq!(res, Decimal::from_i128_with_scale(250, 2).unwrap());

        // Value in the middle of X3-X4 should give y = (700+1_000_000_000) / 2
        let x = Decimal::from_i128_with_scale(7, 2).unwrap();
        let res = calc_y(x, &curve).unwrap();
        assert_eq!(
            res,
            Decimal::from_i128_with_scale((700 + 1_000_000_000) / 2, 2).unwrap()
        );
    }
}
