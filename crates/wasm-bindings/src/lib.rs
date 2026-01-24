//! WASM bindings for Keystone financial computation.

use precision_core::{Decimal, RoundingMode};
use wasm_bindgen::prelude::*;

fn parse_decimal(s: &str) -> Result<Decimal, JsError> {
    s.parse().map_err(|e| JsError::new(&format!("{}", e)))
}

fn to_result(d: Result<Decimal, precision_core::ArithmeticError>) -> Result<String, JsError> {
    d.map(|v| v.to_string())
        .map_err(|e| JsError::new(&format!("{}", e)))
}

// ============================================================================
// Core Arithmetic
// ============================================================================

#[wasm_bindgen]
pub fn add(a: &str, b: &str) -> Result<String, JsError> {
    let a = parse_decimal(a)?;
    let b = parse_decimal(b)?;
    to_result(a.try_add(b))
}

#[wasm_bindgen]
pub fn subtract(a: &str, b: &str) -> Result<String, JsError> {
    let a = parse_decimal(a)?;
    let b = parse_decimal(b)?;
    to_result(a.try_sub(b))
}

#[wasm_bindgen]
pub fn multiply(a: &str, b: &str) -> Result<String, JsError> {
    let a = parse_decimal(a)?;
    let b = parse_decimal(b)?;
    to_result(a.try_mul(b))
}

#[wasm_bindgen]
pub fn divide(a: &str, b: &str) -> Result<String, JsError> {
    let a = parse_decimal(a)?;
    let b = parse_decimal(b)?;
    to_result(a.try_div(b))
}

#[wasm_bindgen]
pub fn round(value: &str, decimal_places: u32, mode: &str) -> Result<String, JsError> {
    let v = parse_decimal(value)?;
    let rounding_mode = match mode {
        "down" => RoundingMode::Down,
        "up" => RoundingMode::Up,
        "toward_zero" | "truncate" => RoundingMode::TowardZero,
        "away_from_zero" => RoundingMode::AwayFromZero,
        "half_even" | "bankers" => RoundingMode::HalfEven,
        "half_up" => RoundingMode::HalfUp,
        "half_down" => RoundingMode::HalfDown,
        _ => return Err(JsError::new(&format!("unknown rounding mode: {}", mode))),
    };
    Ok(v.round(decimal_places, rounding_mode).to_string())
}

#[wasm_bindgen]
pub fn abs(value: &str) -> Result<String, JsError> {
    let v = parse_decimal(value)?;
    Ok(v.abs().to_string())
}

#[wasm_bindgen]
pub fn negate(value: &str) -> Result<String, JsError> {
    let v = parse_decimal(value)?;
    Ok((-v).to_string())
}

#[wasm_bindgen]
pub fn min(a: &str, b: &str) -> Result<String, JsError> {
    let a = parse_decimal(a)?;
    let b = parse_decimal(b)?;
    Ok(a.min(b).to_string())
}

#[wasm_bindgen]
pub fn max(a: &str, b: &str) -> Result<String, JsError> {
    let a = parse_decimal(a)?;
    let b = parse_decimal(b)?;
    Ok(a.max(b).to_string())
}

#[wasm_bindgen]
pub fn compare(a: &str, b: &str) -> Result<i32, JsError> {
    let a = parse_decimal(a)?;
    let b = parse_decimal(b)?;
    Ok(match a.cmp(&b) {
        core::cmp::Ordering::Less => -1,
        core::cmp::Ordering::Equal => 0,
        core::cmp::Ordering::Greater => 1,
    })
}

// ============================================================================
// Financial Calculations
// ============================================================================

#[wasm_bindgen]
pub fn simple_interest(principal: &str, rate: &str, periods: &str) -> Result<String, JsError> {
    let p = parse_decimal(principal)?;
    let r = parse_decimal(rate)?;
    let t = parse_decimal(periods)?;
    to_result(financial_calc::simple_interest(p, r, t))
}

#[wasm_bindgen]
pub fn compound_interest(
    principal: &str,
    rate: &str,
    compounds_per_period: u32,
    periods: u32,
) -> Result<String, JsError> {
    let p = parse_decimal(principal)?;
    let r = parse_decimal(rate)?;
    to_result(financial_calc::compound_interest(
        p,
        r,
        compounds_per_period,
        periods,
    ))
}

#[wasm_bindgen]
pub fn effective_annual_rate(
    nominal_rate: &str,
    compounds_per_year: u32,
) -> Result<String, JsError> {
    let r = parse_decimal(nominal_rate)?;
    to_result(financial_calc::effective_annual_rate(r, compounds_per_year))
}

#[wasm_bindgen]
pub fn percentage_of(value: &str, percentage: &str) -> Result<String, JsError> {
    let v = parse_decimal(value)?;
    let p = parse_decimal(percentage)?;
    to_result(financial_calc::percentage_of(v, p))
}

#[wasm_bindgen]
pub fn percentage_change(old_value: &str, new_value: &str) -> Result<String, JsError> {
    let old = parse_decimal(old_value)?;
    let new = parse_decimal(new_value)?;
    to_result(financial_calc::percentage_change(old, new))
}

#[wasm_bindgen]
pub fn basis_points_to_decimal(bps: &str) -> Result<String, JsError> {
    let b = parse_decimal(bps)?;
    to_result(financial_calc::basis_points_to_decimal(b))
}

#[wasm_bindgen]
pub fn future_value(present_value: &str, rate: &str, periods: u32) -> Result<String, JsError> {
    let pv = parse_decimal(present_value)?;
    let r = parse_decimal(rate)?;
    to_result(financial_calc::future_value(pv, r, periods))
}

#[wasm_bindgen]
pub fn present_value(future_value: &str, rate: &str, periods: u32) -> Result<String, JsError> {
    let fv = parse_decimal(future_value)?;
    let r = parse_decimal(rate)?;
    to_result(financial_calc::present_value(fv, r, periods))
}

// ============================================================================
// Risk Metrics
// ============================================================================

#[wasm_bindgen]
pub fn health_factor(
    collateral_value: &str,
    debt_value: &str,
    liquidation_threshold: &str,
) -> Result<String, JsError> {
    let c = parse_decimal(collateral_value)?;
    let d = parse_decimal(debt_value)?;
    let t = parse_decimal(liquidation_threshold)?;
    to_result(risk_metrics::health_factor(c, d, t))
}

#[wasm_bindgen]
pub fn is_healthy(
    collateral_value: &str,
    debt_value: &str,
    liquidation_threshold: &str,
    min_health_factor: &str,
) -> Result<bool, JsError> {
    let c = parse_decimal(collateral_value)?;
    let d = parse_decimal(debt_value)?;
    let t = parse_decimal(liquidation_threshold)?;
    let m = parse_decimal(min_health_factor)?;
    risk_metrics::is_healthy(c, d, t, m).map_err(|e| JsError::new(&format!("{}", e)))
}

#[wasm_bindgen]
pub fn liquidation_price(
    collateral_amount: &str,
    debt_value: &str,
    liquidation_threshold: &str,
) -> Result<String, JsError> {
    let c = parse_decimal(collateral_amount)?;
    let d = parse_decimal(debt_value)?;
    let t = parse_decimal(liquidation_threshold)?;
    to_result(risk_metrics::liquidation_price(c, d, t))
}

#[wasm_bindgen]
pub fn max_borrowable(
    collateral_value: &str,
    max_ltv: &str,
    current_debt: &str,
) -> Result<String, JsError> {
    let c = parse_decimal(collateral_value)?;
    let m = parse_decimal(max_ltv)?;
    let d = parse_decimal(current_debt)?;
    to_result(risk_metrics::max_borrowable(c, m, d))
}

#[wasm_bindgen]
pub fn loan_to_value(debt_value: &str, collateral_value: &str) -> Result<String, JsError> {
    let d = parse_decimal(debt_value)?;
    let c = parse_decimal(collateral_value)?;
    to_result(risk_metrics::loan_to_value(d, c))
}

#[wasm_bindgen]
pub fn utilization_rate(total_borrows: &str, total_supply: &str) -> Result<String, JsError> {
    let b = parse_decimal(total_borrows)?;
    let s = parse_decimal(total_supply)?;
    to_result(risk_metrics::utilization_rate(b, s))
}

#[wasm_bindgen]
pub fn available_liquidity(total_supply: &str, total_borrows: &str) -> Result<String, JsError> {
    let s = parse_decimal(total_supply)?;
    let b = parse_decimal(total_borrows)?;
    to_result(risk_metrics::available_liquidity(s, b))
}

#[wasm_bindgen]
pub fn collateral_ratio(collateral_value: &str, debt_value: &str) -> Result<String, JsError> {
    let c = parse_decimal(collateral_value)?;
    let d = parse_decimal(debt_value)?;
    to_result(risk_metrics::collateral_ratio(c, d))
}
