#![no_main]
sp1_zkvm::entrypoint!(main);

use precision_core::{Decimal, RoundingMode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum FinancialOperation {
    HealthFactor {
        collateral_value: i128,
        debt_value: i128,
        liquidation_threshold_bps: i128,
    },
    CompoundInterest {
        principal: i128,
        rate_bps: i128,
        periods: u32,
    },
    SwapOutput {
        reserve_in: i128,
        reserve_out: i128,
        amount_in: i128,
        fee_bps: i128,
    },
    LiquidationPrice {
        collateral_amount: i128,
        debt_value: i128,
        liquidation_threshold_bps: i128,
    },
    SharePrice {
        total_assets: i128,
        total_supply: i128,
    },
}

#[derive(Serialize, Deserialize)]
pub struct FinancialResult {
    pub value: i128,
    pub scale: u32,
}

fn main() {
    let operation: FinancialOperation = sp1_zkvm::io::read();
    let result = execute_operation(operation);
    sp1_zkvm::io::commit(&result);
}

fn execute_operation(op: FinancialOperation) -> FinancialResult {
    match op {
        FinancialOperation::HealthFactor {
            collateral_value,
            debt_value,
            liquidation_threshold_bps,
        } => calculate_health_factor(collateral_value, debt_value, liquidation_threshold_bps),

        FinancialOperation::CompoundInterest {
            principal,
            rate_bps,
            periods,
        } => calculate_compound_interest(principal, rate_bps, periods),

        FinancialOperation::SwapOutput {
            reserve_in,
            reserve_out,
            amount_in,
            fee_bps,
        } => calculate_swap_output(reserve_in, reserve_out, amount_in, fee_bps),

        FinancialOperation::LiquidationPrice {
            collateral_amount,
            debt_value,
            liquidation_threshold_bps,
        } => calculate_liquidation_price(collateral_amount, debt_value, liquidation_threshold_bps),

        FinancialOperation::SharePrice {
            total_assets,
            total_supply,
        } => calculate_share_price(total_assets, total_supply),
    }
}

fn calculate_health_factor(
    collateral_value: i128,
    debt_value: i128,
    liquidation_threshold_bps: i128,
) -> FinancialResult {
    if debt_value == 0 {
        return FinancialResult {
            value: i128::MAX,
            scale: 18,
        };
    }

    let collateral = Decimal::from(collateral_value);
    let debt = Decimal::from(debt_value);
    let threshold = Decimal::from(liquidation_threshold_bps)
        .checked_div(Decimal::from(10_000i64))
        .unwrap_or(Decimal::ZERO);

    let weighted_collateral = collateral.checked_mul(threshold).unwrap_or(Decimal::ZERO);
    let health_factor = weighted_collateral
        .checked_div(debt)
        .unwrap_or(Decimal::ZERO);

    decimal_to_result(health_factor)
}

fn calculate_compound_interest(principal: i128, rate_bps: i128, periods: u32) -> FinancialResult {
    let p = Decimal::from(principal);
    let rate = Decimal::from(rate_bps)
        .checked_div(Decimal::from(10_000i64))
        .unwrap_or(Decimal::ZERO);

    let one_plus_rate = Decimal::ONE.checked_add(rate).unwrap_or(Decimal::ONE);

    let mut result = p;
    for _ in 0..periods {
        result = result.checked_mul(one_plus_rate).unwrap_or(Decimal::MAX);
    }

    decimal_to_result(result)
}

fn calculate_swap_output(
    reserve_in: i128,
    reserve_out: i128,
    amount_in: i128,
    fee_bps: i128,
) -> FinancialResult {
    let r_in = Decimal::from(reserve_in);
    let r_out = Decimal::from(reserve_out);
    let a_in = Decimal::from(amount_in);
    let fee = Decimal::from(fee_bps)
        .checked_div(Decimal::from(10_000i64))
        .unwrap_or(Decimal::ZERO);

    let fee_multiplier = Decimal::ONE.checked_sub(fee).unwrap_or(Decimal::ONE);
    let effective_in = a_in.checked_mul(fee_multiplier).unwrap_or(Decimal::ZERO);

    let numerator = effective_in.checked_mul(r_out).unwrap_or(Decimal::ZERO);
    let denominator = r_in.checked_add(effective_in).unwrap_or(Decimal::ONE);
    let amount_out = numerator
        .checked_div(denominator)
        .unwrap_or(Decimal::ZERO);

    decimal_to_result(amount_out)
}

fn calculate_liquidation_price(
    collateral_amount: i128,
    debt_value: i128,
    liquidation_threshold_bps: i128,
) -> FinancialResult {
    if collateral_amount == 0 {
        return FinancialResult { value: 0, scale: 18 };
    }

    let amount = Decimal::from(collateral_amount);
    let debt = Decimal::from(debt_value);
    let threshold = Decimal::from(liquidation_threshold_bps)
        .checked_div(Decimal::from(10_000i64))
        .unwrap_or(Decimal::ONE);

    let denominator = amount.checked_mul(threshold).unwrap_or(Decimal::ONE);
    let liq_price = debt.checked_div(denominator).unwrap_or(Decimal::ZERO);

    decimal_to_result(liq_price)
}

fn calculate_share_price(total_assets: i128, total_supply: i128) -> FinancialResult {
    if total_supply == 0 {
        return FinancialResult {
            value: 1_000_000_000_000_000_000,
            scale: 18,
        };
    }

    let assets = Decimal::from(total_assets);
    let supply = Decimal::from(total_supply);
    let price = assets.checked_div(supply).unwrap_or(Decimal::ONE);

    decimal_to_result(price)
}

fn decimal_to_result(value: Decimal) -> FinancialResult {
    let scale_factor = Decimal::from(1_000_000_000_000_000_000i64);
    let scaled = value
        .checked_mul(scale_factor)
        .unwrap_or(Decimal::MAX)
        .round(0, RoundingMode::TowardZero);
    let (mantissa, _) = scaled.to_parts();

    FinancialResult {
        value: mantissa,
        scale: 18,
    }
}
