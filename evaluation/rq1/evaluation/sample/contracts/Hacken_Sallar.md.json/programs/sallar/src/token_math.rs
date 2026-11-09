use anchor_lang::error::Error;

use crate::utils::{convert_f64_to_u64, convert_u64_to_f64};

/// In this module for numerical calculations, we have carefully considered the appropriate data types to use for different types of calculations.
/// To ensure accurate and efficient computations, we have employed a strategy that utilizes f64 for non-financial calculations and u64 for financial calculations.
/// This approach takes into account the performance, precision, compliance, readability, maintainability, portability, and robustness requirements of the calculations.
///
/// Using f64 for numerical calculations allows for efficient and fast computations due to its native floating-point implementation in Rust.
/// f64 can accurately represent approximately 15-17 significant decimal digits, which provides a high level of precision for calculations.
/// To ensure accurate representation and manipulation of financial amounts with strict adherence to rounding rules and precision requirements,
/// we utilize u64 as the data type for storing and processing financial results as much as possible and f64 only for the final part
/// of some of the calculations where it is strictly required. By using f64 for numerical calculations and u64 for most of the financial amounts,
/// we strike a balance between performance and precision, ensuring efficient computations while maintaining accuracy and compliance in financial calculations.
/// In the most cases f64 is enough to provide full precision.
/// In the other rare cases some small lack of precision is introduced but it influences results in a very limited way (the inaccuracy is very low).
///
/// The accuracy and compliance of the calculations were thoroughly verified using Python scripts,
/// which were also used to generate comprehensive test data to ensure the correctness and reliability of the implementation.
/// This approach combines the performance benefits of f64 with the precise representation of u64,
/// while also utilizing Python for both verification and test data generation to ensure the accuracy and reliability of the calculations in our Rust module.
///
/// Total supply for SPL Token cannot exceed u64 range in Solana.
/// There are no operations in this contract that would exceed total supply so this is why usage of u64 is safe here.

/// Set of functions and constants defining the most important math functions used by the contract.

pub const TOKEN_AMOUNT_SCALING_FACTOR: u64 = 1_000;
pub const DUSTS_PER_BLOCK: u64 = 2_000_000_000 * TOKEN_AMOUNT_SCALING_FACTOR;

const MAX_BLOCK_INDEX: u64 = 2_600_000;

const FIRST_BP: f64 = 20.0 * (TOKEN_AMOUNT_SCALING_FACTOR as f64);
const REDUCTION_INVERSE: f64 = 0.99999430521433;

const MIN_REQUIRED_STAKE_FOR_BOTTOM_BLOCK_DUST: u64 =
    2_000_000_000 * TOKEN_AMOUNT_SCALING_FACTOR as u64;

const MAX_BOTTOM_BOOST: f64 = 60.0;
const BOTTOM_BOOST_REDUCTION: f64 = 0.999997999992;

const MIN_TOP_BOOST: f64 = 0.5;
const TOP_BOOST_REDUCTION: f64 = 1.000004498927;

const TOP_FIRST_BOOSTED_BLOCK: f64 = 250.0;
const TOP_BP_WITHOUT_BOOST: u64 = 1 * TOKEN_AMOUNT_SCALING_FACTOR;

fn dust_to_staking_sallar(dusts: u64) -> u64 {
    // 1 dust = 1e-8 sallar, only the whole sallar will be staked
    // truncation of the decimal part is intentional
    dusts / (100_000_000)
}

fn calculate_bp_reduction_factor(block_index: u64) -> Result<f64, Error> {
    Ok(REDUCTION_INVERSE.powf(convert_u64_to_f64(block_index - 1)?))
}

pub fn calculate_max_bp(block_index: u64) -> Result<f64, Error> {
    let bp_reduction_factor = calculate_bp_reduction_factor(block_index)?;

    Ok((FIRST_BP / bp_reduction_factor).round())
}

pub fn calculate_dust_per_bp(block_index: u64) -> Result<f64, Error> {
    let max_bp = calculate_max_bp(block_index)?;
    Ok(convert_u64_to_f64(DUSTS_PER_BLOCK)? / max_bp)
}

fn calculate_top_block_max_boost(block_index: u64) -> Result<u64, Error> {
    let exp = convert_u64_to_f64(block_index)? - TOP_FIRST_BOOSTED_BLOCK;
    let pow = TOP_BOOST_REDUCTION.powf(exp);

    let base_boost = MIN_TOP_BOOST * pow;
    let rounded_boost;

    if base_boost < 1e+2 {
        rounded_boost = convert_f64_to_u64(base_boost.round())?;
    } else if base_boost < 1e+3 {
        rounded_boost = convert_f64_to_u64(base_boost * 0.1)? * 10;
    } else {
        rounded_boost = convert_f64_to_u64(base_boost * 0.01)? * 100;
    }

    Ok(rounded_boost)
}

fn calculate_base_bp_for_given_boost(boost: u64) -> u64 {
    1 + boost
}

fn calculate_top_bp(boost: u64) -> Result<u64, Error> {
    Ok(TOKEN_AMOUNT_SCALING_FACTOR * calculate_base_bp_for_given_boost(boost))
}

pub fn calculate_top_bp_with_boost(block_index: u64) -> Result<u64, Error> {
    let boost = calculate_top_block_max_boost(block_index)?;

    Ok(calculate_top_bp(boost)?)
}

fn calculate_bottom_block_max_boost(block_index: u64) -> Result<u64, Error> {
    let base_boost = MAX_BOTTOM_BOOST
        * BOTTOM_BOOST_REDUCTION.powf(convert_u64_to_f64(MAX_BLOCK_INDEX - block_index)?);

    convert_f64_to_u64(base_boost.round())
}

fn calculate_bottom_bp(user_wallet_balance: u64, boost: u64) -> u64 {
    calculate_base_bp_for_given_boost(boost) * (dust_to_staking_sallar(user_wallet_balance))
}

pub fn calculate_bottom_bp_without_boost(user_wallet_balance: u64) -> u64 {
    calculate_bottom_bp(user_wallet_balance, 0)
}

pub fn calculate_bottom_bp_with_boost(
    block_index: u64,
    user_wallet_balance: u64,
) -> Result<u64, Error> {
    let boost = calculate_bottom_block_max_boost(block_index)?;

    Ok(calculate_bottom_bp(user_wallet_balance, boost))
}

pub fn calculate_single_reward(bp: u64, dust_per_bp: f64) -> Result<u64, Error> {
    Ok(convert_f64_to_u64(
        (convert_u64_to_f64(bp)? * dust_per_bp).round(),
    )?)
}

/// The function calculates parts of the reward separately for requests with boost and without boost.
/// They are kept separate from each other, and the reason they are summed up in the end
/// is to consolidate them into a single transfer, instead of two separate transfers for each reward part.
/// However, the calculation is intentionally done this way, as the parts are semantically separated.
fn calculate_user_reward(
    user_request_without_boost: u8,
    user_request_with_boost: u8,
    parts_without_boost: u64,
    parts_with_boost: u64,
    dust_per_bp: f64,
) -> Result<(u64, u64), Error> {
    let amount_without_boost = (user_request_without_boost as u64)
        * calculate_single_reward(parts_without_boost, dust_per_bp)?;
    let amount_with_boost =
        (user_request_with_boost as u64) * calculate_single_reward(parts_with_boost, dust_per_bp)?;

    let total_bp = ((user_request_without_boost as u64) * parts_without_boost)
        + ((user_request_with_boost as u64) * parts_with_boost);
    let summary_amount = amount_without_boost + amount_with_boost;

    Ok((total_bp, summary_amount))
}

pub fn calculate_user_reward_bottom_block(
    user_request_without_boost: u8,
    user_request_with_boost: u8,
    parts_without_boost: u64,
    parts_with_boost: u64,
    dust_per_bp: f64,
    user_wallet_balance: u64,
) -> Result<(u64, u64), Error> {
    if user_wallet_balance < MIN_REQUIRED_STAKE_FOR_BOTTOM_BLOCK_DUST {
        return Ok((0, 0));
    }

    Ok(calculate_user_reward(
        user_request_without_boost,
        user_request_with_boost,
        parts_without_boost,
        parts_with_boost,
        dust_per_bp,
    )?)
}

pub fn calculate_user_reward_top_block(
    user_request_without_boost: u8,
    user_request_with_boost: u8,
    parts_with_boost: u64,
    dust_per_bp: f64,
) -> Result<(u64, u64), Error> {
    Ok(calculate_user_reward(
        user_request_without_boost,
        user_request_with_boost,
        TOP_BP_WITHOUT_BOOST,
        parts_with_boost,
        dust_per_bp,
    )?)
}

#[cfg(test)]
mod tests {
    use std::{error::Error as standardError, fs::File};

    use super::*;

    #[test]
    fn generate_csv_report_top_block() -> Result<(), Box<dyn standardError>> {
        let file = File::open("./top_block_reports/dustAndBlockPartReportTop.csv")?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.records() {
            let record = result?;

            let block_index = record.get(0).unwrap().parse::<u64>().unwrap();
            let dust_without_boost_expected = record.get(1).unwrap().parse::<u64>().unwrap();
            let dust_with_boost_expected = record.get(2).unwrap().parse::<u64>().unwrap();
            let bp_with_boost_expected = record.get(3).unwrap().parse::<u64>().unwrap();
            let dust_per_bp_expected = record.get(4).unwrap().parse::<f64>().unwrap();
            let until_block_index = record.get(5).unwrap().parse::<u64>().unwrap();

            let indexes = vec![block_index, until_block_index];

            for index in indexes {
                let sallar_per_bp = calculate_dust_per_bp(index).unwrap();
                let top_block_bp_with_boost = calculate_top_bp_with_boost(index).unwrap();

                let (_, top_block_dust_without_boost) =
                    calculate_user_reward_top_block(1, 0, top_block_bp_with_boost, sallar_per_bp)
                        .unwrap();
                let (_, top_block_dust_with_boost) =
                    calculate_user_reward_top_block(0, 1, top_block_bp_with_boost, sallar_per_bp)
                        .unwrap();

                let dust_per_bp = calculate_dust_per_bp(index).unwrap();

                assert_eq!(
                    bp_with_boost_expected.to_string(),
                    top_block_bp_with_boost.to_string()
                );
                assert_eq!(
                    dust_with_boost_expected.to_string(),
                    top_block_dust_with_boost.to_string()
                );
                assert_eq!(
                    dust_without_boost_expected.to_string(),
                    top_block_dust_without_boost.to_string()
                );
                assert_eq!(dust_per_bp_expected.to_string(), dust_per_bp.to_string());
            }
        }

        Ok(())
    }

    #[test]
    fn generate_csv_report_bottom_block() -> Result<(), Box<dyn standardError>> {
        let file = File::open("./bottom_block_reports/dustAndBlockPartReportBottom.csv")?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.records() {
            let record = result?;

            let block_index = record.get(0).unwrap().parse::<u64>().unwrap();
            let balance = record.get(1).unwrap().parse::<u64>().unwrap();

            let sallar_without_boost_expected = record.get(2).unwrap().parse::<u64>().unwrap();
            let bp_without_boost_expected = record.get(3).unwrap().parse::<u64>().unwrap();
            let sallar_with_boost_expected = record.get(4).unwrap().parse::<u64>().unwrap();
            let bp_with_boost_expected = record.get(5).unwrap().parse::<u64>().unwrap();
            let sallar_per_bp_expected = record.get(6).unwrap().parse::<f64>().unwrap();

            let bottom_block_bp_with_boost =
                calculate_bottom_bp_with_boost(block_index, balance).unwrap();
            let bottom_block_bp_without_boost = calculate_bottom_bp_without_boost(balance);
            let sallar_per_bp = calculate_dust_per_bp(block_index).unwrap();

            let (_, bottom_block_staking_dust_without_boost) = calculate_user_reward_bottom_block(
                1,
                0,
                bottom_block_bp_without_boost,
                bottom_block_bp_with_boost,
                sallar_per_bp,
                balance,
            )
            .unwrap();
            let (_, bottom_block_staking_dust_with_boost) = calculate_user_reward_bottom_block(
                0,
                1,
                bottom_block_bp_without_boost,
                bottom_block_bp_with_boost,
                sallar_per_bp,
                balance,
            )
            .unwrap();

            let bp_without_boost = calculate_bottom_bp_without_boost(balance);

            assert_eq!(
                bp_without_boost_expected.to_string(),
                bp_without_boost.to_string()
            );
            assert_eq!(
                bp_with_boost_expected.to_string(),
                bottom_block_bp_with_boost.to_string()
            );

            assert_eq!(
                sallar_per_bp_expected.to_string(),
                sallar_per_bp.to_string()
            );

            assert_eq!(
                sallar_without_boost_expected.to_string(),
                bottom_block_staking_dust_without_boost.to_string(),
                "block_index: {}",
                block_index
            );
            assert_eq!(
                sallar_with_boost_expected.to_string(),
                bottom_block_staking_dust_with_boost.to_string(),
                "block_index: {}",
                block_index
            );
        }

        Ok(())
    }

    #[test]
    pub fn calculate_user_reward_top_block_test() -> Result<(), Box<dyn standardError>> {
        let file = File::open("./top_block_reports/topBlockTransferTestData.csv")?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.records() {
            let record = result?;

            let block_index = record.get(0).unwrap().parse::<u64>().unwrap();
            let user_request_without_boost = record.get(1).unwrap().parse::<u64>().unwrap();
            let user_request_with_boost = record.get(2).unwrap().parse::<u64>().unwrap();

            let reward_dust_expected = record.get(3).unwrap().parse::<u64>().unwrap();

            let top_block_bp_with_boost = calculate_top_bp_with_boost(block_index).unwrap();
            let dust_per_bp = calculate_dust_per_bp(block_index).unwrap();
            let (_, reward_dust) = calculate_user_reward_top_block(
                user_request_without_boost as u8,
                user_request_with_boost as u8,
                top_block_bp_with_boost,
                dust_per_bp,
            )
            .unwrap();

            assert_eq!(reward_dust_expected.to_string(), reward_dust.to_string());
        }

        Ok(())
    }

    #[test]
    pub fn calculate_user_reward_bottom_block_test() -> Result<(), Box<dyn standardError>> {
        let file = File::open("./bottom_block_reports/bottomBlockTransferTestData.csv")?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.records() {
            let record = result?;

            let block_index = record.get(0).unwrap().parse::<u64>().unwrap();
            let user_request_without_boost = record.get(1).unwrap().parse::<u64>().unwrap();
            let user_request_with_boost = record.get(2).unwrap().parse::<u64>().unwrap();

            let user_wallet_balance = record.get(3).unwrap().parse::<u64>().unwrap();
            let reward_dust_expected = record.get(4).unwrap().parse::<u64>().unwrap();
            let bp_expected = record.get(5).unwrap().parse::<u64>().unwrap();

            let bottom_block_bp_without_boost =
                calculate_bottom_bp_without_boost(user_wallet_balance);
            let bottom_block_bp_with_boost =
                calculate_bottom_bp_with_boost(block_index, user_wallet_balance).unwrap();
            let dust_per_bp = calculate_dust_per_bp(block_index).unwrap();

            let (_, reward_dust) = calculate_user_reward_bottom_block(
                user_request_without_boost as u8,
                user_request_with_boost as u8,
                bottom_block_bp_without_boost,
                bottom_block_bp_with_boost,
                dust_per_bp,
                user_wallet_balance,
            )
            .unwrap();
            let bp = bottom_block_bp_without_boost * user_request_without_boost
                + bottom_block_bp_with_boost * user_request_with_boost;

            assert_eq!(bp_expected.to_string(), bp.to_string());
            assert_eq!(reward_dust_expected.to_string(), reward_dust.to_string());
        }

        Ok(())
    }
}
