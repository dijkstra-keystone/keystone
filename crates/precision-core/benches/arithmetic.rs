//! Benchmarks for precision arithmetic operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use precision_core::{Decimal, RoundingMode};

fn arithmetic_benchmarks(c: &mut Criterion) {
    let a = Decimal::new(123456789, 6);
    let b = Decimal::new(987654321, 6);

    c.bench_function("addition", |bench| {
        bench.iter(|| black_box(a.checked_add(b)))
    });

    c.bench_function("subtraction", |bench| {
        bench.iter(|| black_box(b.checked_sub(a)))
    });

    let x = Decimal::new(123456, 3);
    let y = Decimal::new(789012, 3);

    c.bench_function("multiplication", |bench| {
        bench.iter(|| black_box(x.checked_mul(y)))
    });

    c.bench_function("division", |bench| {
        bench.iter(|| black_box(a.checked_div(Decimal::new(12345, 3))))
    });
}

fn rounding_benchmarks(c: &mut Criterion) {
    let a = Decimal::new(123456789, 8);

    c.bench_function("round_half_even", |bench| {
        bench.iter(|| black_box(a.round(2, RoundingMode::HalfEven)))
    });

    c.bench_function("round_half_up", |bench| {
        bench.iter(|| black_box(a.round(2, RoundingMode::HalfUp)))
    });

    c.bench_function("floor", |bench| bench.iter(|| black_box(a.floor())));

    c.bench_function("ceil", |bench| bench.iter(|| black_box(a.ceil())));
}

fn comparison_benchmarks(c: &mut Criterion) {
    let a = Decimal::new(123456789, 6);
    let b = Decimal::new(123456790, 6);

    c.bench_function("comparison", |bench| bench.iter(|| black_box(a.cmp(&b))));

    c.bench_function("min", |bench| bench.iter(|| black_box(a.min(b))));

    c.bench_function("max", |bench| bench.iter(|| black_box(a.max(b))));
}

fn parsing_benchmarks(c: &mut Criterion) {
    let s = "123456.789012345";

    c.bench_function("parse", |bench| {
        bench.iter(|| black_box(s.parse::<Decimal>()))
    });

    let a = Decimal::new(123456789012345, 9);

    c.bench_function("to_string", |bench| bench.iter(|| black_box(a.to_string())));
}

fn defi_benchmarks(c: &mut Criterion) {
    let collateral = Decimal::from(10000i64);
    let debt = Decimal::from(5000i64);
    let threshold = Decimal::new(80, 2);

    c.bench_function("health_factor", |bench| {
        bench.iter(|| {
            let weighted = collateral.checked_mul(threshold).unwrap();
            black_box(weighted.checked_div(debt))
        })
    });

    let rate = Decimal::new(5, 2);
    let base = Decimal::ONE.checked_add(rate).unwrap();

    c.bench_function("compound_12_periods", |bench| {
        bench.iter(|| {
            let mut result = Decimal::ONE;
            for _ in 0..12 {
                result = result.checked_mul(base).unwrap();
            }
            black_box(result)
        })
    });
}

criterion_group!(
    benches,
    arithmetic_benchmarks,
    rounding_benchmarks,
    comparison_benchmarks,
    parsing_benchmarks,
    defi_benchmarks,
);

criterion_main!(benches);
