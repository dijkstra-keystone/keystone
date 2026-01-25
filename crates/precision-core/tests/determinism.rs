//! Cross-platform determinism verification tests.
//!
//! These tests verify that calculations produce identical results across platforms
//! by checking against known test vectors.

extern crate alloc;

use precision_core::{Decimal, RoundingMode};

/// Test vectors with known outputs that must match across all platforms.
/// Format: (input_a, input_b, expected_sum, expected_product, expected_quotient)
const ARITHMETIC_VECTORS: &[(&str, &str, &str, &str, &str)] = &[
    ("1.0", "2.0", "3.0", "2.0", "0.5"),
    ("0.1", "0.2", "0.3", "0.02", "0.5"),
    (
        "123.456",
        "789.012",
        "912.468",
        "97408.265472",
        "0.1564691031315113078128089307",
    ),
    ("-100.5", "50.25", "-50.25", "-5050.125", "-2"),
    (
        "0.000001",
        "1000000",
        "1000000.000001",
        "1",
        "0.000000000001",
    ),
    (
        "99999.99999",
        "0.00001",
        "100000",
        "0.9999999999",
        "9999999999",
    ),
];

#[test]
fn arithmetic_determinism() {
    for (a_str, b_str, sum_str, product_str, quotient_str) in ARITHMETIC_VECTORS {
        let a: Decimal = a_str.parse().unwrap();
        let b: Decimal = b_str.parse().unwrap();
        let expected_sum: Decimal = sum_str.parse().unwrap();
        let expected_product: Decimal = product_str.parse().unwrap();

        let actual_sum = a.checked_add(b).unwrap();
        let actual_product = a.checked_mul(b).unwrap();

        assert_eq!(
            actual_sum, expected_sum,
            "Sum mismatch: {} + {} = {} (expected {})",
            a_str, b_str, actual_sum, sum_str
        );

        assert_eq!(
            actual_product, expected_product,
            "Product mismatch: {} * {} = {} (expected {})",
            a_str, b_str, actual_product, product_str
        );

        if !b.is_zero() {
            let expected_quotient: Decimal = quotient_str.parse().unwrap();
            let actual_quotient = a.checked_div(b).unwrap();
            let diff = (actual_quotient - expected_quotient).abs();
            assert!(
                diff < Decimal::new(1, 15),
                "Quotient mismatch: {} / {} = {} (expected {}, diff = {})",
                a_str,
                b_str,
                actual_quotient,
                quotient_str,
                diff
            );
        }
    }
}

/// Test vectors for rounding operations.
/// Format: (input, scale, half_even_result, half_up_result)
const ROUNDING_VECTORS: &[(&str, u32, &str, &str)] = &[
    ("2.5", 0, "2", "3"),
    ("3.5", 0, "4", "4"),
    ("2.25", 1, "2.2", "2.3"),
    ("2.35", 1, "2.4", "2.4"),
    ("-2.5", 0, "-2", "-3"),
    ("-3.5", 0, "-4", "-4"),
    ("1.2345", 2, "1.23", "1.23"),
    ("1.2355", 2, "1.24", "1.24"),
    ("1.2350", 2, "1.24", "1.24"),
    ("0.005", 2, "0.00", "0.01"),
    ("0.015", 2, "0.02", "0.02"),
];

#[test]
fn rounding_determinism() {
    for (input_str, scale, half_even_str, half_up_str) in ROUNDING_VECTORS {
        let input: Decimal = input_str.parse().unwrap();
        let expected_half_even: Decimal = half_even_str.parse().unwrap();
        let expected_half_up: Decimal = half_up_str.parse().unwrap();

        let actual_half_even = input.round(*scale, RoundingMode::HalfEven);
        let actual_half_up = input.round(*scale, RoundingMode::HalfUp);

        assert_eq!(
            actual_half_even, expected_half_even,
            "HalfEven mismatch: round({}, {}) = {} (expected {})",
            input_str, scale, actual_half_even, half_even_str
        );

        assert_eq!(
            actual_half_up, expected_half_up,
            "HalfUp mismatch: round({}, {}) = {} (expected {})",
            input_str, scale, actual_half_up, half_up_str
        );
    }
}

/// Test vectors for comparison operations.
const COMPARISON_VECTORS: &[(&str, &str, i8)] = &[
    ("1.0", "2.0", -1),
    ("2.0", "1.0", 1),
    ("1.0", "1.0", 0),
    ("1.00", "1.0", 0),
    ("-1.0", "1.0", -1),
    ("0.0", "-0.0", 0),
    ("0.1", "0.10", 0),
    ("999999999.999999999", "999999999.999999998", 1),
];

#[test]
fn comparison_determinism() {
    for (a_str, b_str, expected_cmp) in COMPARISON_VECTORS {
        let a: Decimal = a_str.parse().unwrap();
        let b: Decimal = b_str.parse().unwrap();

        let actual_cmp = match a.cmp(&b) {
            core::cmp::Ordering::Less => -1i8,
            core::cmp::Ordering::Equal => 0i8,
            core::cmp::Ordering::Greater => 1i8,
        };

        assert_eq!(
            actual_cmp, *expected_cmp,
            "Comparison mismatch: {} cmp {} = {} (expected {})",
            a_str, b_str, actual_cmp, expected_cmp
        );
    }
}

/// DeFi-specific calculation vectors to ensure protocol-matching precision.
const DEFI_VECTORS: &[(&str, &str, &str, &str)] = &[
    // (collateral, debt, threshold, expected_health_factor)
    ("1000", "500", "0.8", "1.6"),
    ("10000", "7500", "0.825", "1.1"),
    ("5000000", "3750000", "0.85", "1.133333333333333333"),
];

#[test]
fn defi_calculation_determinism() {
    for (collateral_str, debt_str, threshold_str, expected_hf_str) in DEFI_VECTORS {
        let collateral: Decimal = collateral_str.parse().unwrap();
        let debt: Decimal = debt_str.parse().unwrap();
        let threshold: Decimal = threshold_str.parse().unwrap();
        let expected_hf: Decimal = expected_hf_str.parse().unwrap();

        let weighted = collateral.checked_mul(threshold).unwrap();
        let actual_hf = weighted.checked_div(debt).unwrap();

        let diff = (actual_hf - expected_hf).abs();
        assert!(
            diff < Decimal::new(1, 15),
            "Health factor mismatch: ({} * {}) / {} = {} (expected {}, diff = {})",
            collateral_str,
            threshold_str,
            debt_str,
            actual_hf,
            expected_hf_str,
            diff
        );
    }
}

#[test]
fn string_roundtrip_determinism() {
    let values = [
        Decimal::ZERO,
        Decimal::ONE,
        Decimal::new(12345, 3),
        Decimal::new(-67890, 5),
    ];

    for &value in &values {
        use alloc::string::ToString;

        let s = value.to_string();
        let parsed: Decimal = s.parse().expect("parse failed");

        assert_eq!(
            value.normalize(),
            parsed.normalize(),
            "String roundtrip failed for {} -> {}",
            value,
            s
        );
    }
}

/// Transcendental function determinism test.
/// Verifies sqrt, exp, and ln produce consistent results across platforms.
/// These tests use approximate equality because transcendental implementations
/// may have minor precision differences, but results must be within acceptable bounds.
const TRANSCENDENTAL_INPUTS: &[&str] =
    &["1.0", "4.0", "2.0", "0.25", "100.0", "0.01", "0.5", "10.0"];

#[test]
fn transcendental_determinism() {
    // Tolerance: 1e-7 (7 decimal places of agreement)
    // Transcendental functions use Taylor series which accumulates error across
    // chained operations like exp(ln(x))
    let tolerance = Decimal::new(1, 7);

    for input_str in TRANSCENDENTAL_INPUTS {
        let input: Decimal = input_str.parse().unwrap();

        // sqrt is exact for perfect squares, approximate otherwise
        let sqrt_result = input.try_sqrt();
        assert!(sqrt_result.is_ok(), "sqrt({}) should succeed", input_str);
        let sqrt = sqrt_result.unwrap();
        // Verify sqrt^2 ≈ input (roundtrip check)
        let squared = sqrt.checked_mul(sqrt).unwrap();
        let diff = (squared - input).abs();
        assert!(
            diff < tolerance,
            "sqrt({})^2 = {} (expected {}, diff = {})",
            input_str,
            squared,
            input,
            diff
        );

        // exp / ln inverse relationship: exp(ln(x)) ≈ x for positive x
        if input > Decimal::ZERO {
            if let Ok(ln_val) = input.try_ln() {
                if let Ok(exp_ln) = ln_val.try_exp() {
                    let diff = (exp_ln - input).abs();
                    assert!(
                        diff < tolerance,
                        "exp(ln({})) = {} (expected {}, diff = {})",
                        input_str,
                        exp_ln,
                        input,
                        diff
                    );
                }
            }
        }

        // ln(exp(x)) ≈ x for small x (where exp doesn't overflow)
        if input.abs() < Decimal::from(5i64) {
            if let Ok(exp_val) = input.try_exp() {
                if let Ok(ln_exp) = exp_val.try_ln() {
                    let diff = (ln_exp - input).abs();
                    assert!(
                        diff < tolerance,
                        "ln(exp({})) = {} (expected {}, diff = {})",
                        input_str,
                        ln_exp,
                        input,
                        diff
                    );
                }
            }
        }
    }
}

/// Binary representation determinism.
/// Verifies that to_parts() produces identical (mantissa, scale) across platforms.
const PARTS_VECTORS: &[(&str, i128, u32)] = &[
    ("0", 0, 0),
    ("1", 1, 0),
    ("-1", -1, 0),
    ("100", 100, 0),
    ("0.01", 1, 2),
    ("123.456", 123456, 3),
    ("-999.999", -999999, 3),
    ("0.000001", 1, 6),
];

#[test]
fn binary_representation_determinism() {
    for (value_str, expected_mantissa, expected_scale) in PARTS_VECTORS {
        let value: Decimal = value_str.parse().unwrap();
        let (mantissa, scale) = value.to_parts();

        assert_eq!(
            mantissa, *expected_mantissa,
            "Mantissa mismatch for {}: got {}, expected {}",
            value_str, mantissa, expected_mantissa
        );
        assert_eq!(
            scale, *expected_scale,
            "Scale mismatch for {}: got {}, expected {}",
            value_str, scale, expected_scale
        );
    }
}

/// Oracle conversion determinism.
/// Verifies oracle module functions produce consistent results.
#[test]
fn oracle_conversion_determinism() {
    use precision_core::oracle::{normalize_oracle_price, OracleDecimals};

    // Chainlink BTC/USD price: $50,000.12345678 with 8 decimals
    let btc_raw = 5000012345678i64;
    let btc_price = normalize_oracle_price(btc_raw, OracleDecimals::Eight).unwrap();
    let expected: Decimal = "50000.12345678".parse().unwrap();
    assert_eq!(
        btc_price, expected,
        "BTC price normalization: {} != {}",
        btc_price, expected
    );

    // USDC amount: 1000.50 with 6 decimals
    let usdc_raw = 1000500000i64;
    let usdc_amount = normalize_oracle_price(usdc_raw, OracleDecimals::Six).unwrap();
    let expected_usdc: Decimal = "1000.5".parse().unwrap();
    assert_eq!(
        usdc_amount.normalize(),
        expected_usdc.normalize(),
        "USDC normalization: {} != {}",
        usdc_amount,
        expected_usdc
    );
}
