# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-alpha.1] - 2026-01-24

### Added

#### precision-core
- `Decimal` type with 128-bit fixed-point arithmetic
- 7 rounding modes: `HalfEven`, `HalfUp`, `HalfDown`, `Up`, `Down`, `TowardZero`, `AwayFromZero`
- Checked arithmetic: `checked_add`, `checked_sub`, `checked_mul`, `checked_div`, `checked_rem`
- Try arithmetic with explicit errors: `try_add`, `try_sub`, `try_mul`, `try_div`
- Saturating arithmetic: `saturating_add`, `saturating_sub`, `saturating_mul`
- Comparison and ordering
- String parsing and display
- Serde serialization support
- Postcard binary serialization
- Tolerance comparisons: `approx_eq`, `approx_eq_relative`, `within_percentage`, `within_basis_points`
- Cross-platform determinism verification tests

#### financial-calc
- `simple_interest(principal, rate, time)`
- `compound_interest(principal, rate, periods_per_year, years)`
- `effective_annual_rate(nominal_rate, periods)`
- `future_value(present_value, rate, periods)`
- `present_value(future_value, rate, periods)`
- `net_present_value(rate, cash_flows)`
- `percentage_of(value, percent)`
- `percentage_change(old_value, new_value)`
- `basis_points_to_decimal(bps)`

#### risk-metrics
- `health_factor(collateral, debt, threshold)`
- `is_healthy(collateral, debt, threshold)`
- `collateral_ratio(collateral, debt)`
- `liquidation_price(collateral_amount, debt, threshold)`
- `liquidation_threshold(collateral_value, debt, health_factor)`
- `max_borrowable(collateral, threshold, min_health_factor)`
- `loan_to_value(debt, collateral)`
- `utilization_rate(total_borrows, total_liquidity)`
- `available_liquidity(total_liquidity, total_borrows)`

#### keystone-wasm
- Complete WASM bindings for all arithmetic operations
- Bindings for all financial-calc functions
- Bindings for all risk-metrics functions
- String-based API for JavaScript interop
- ~97KB optimized release build

### Technical Details
- `no_std` compatible core
- `#![forbid(unsafe_code)]` throughout
- 100+ unit and property-based tests
- Criterion benchmarks
- CI pipeline with multi-platform testing
