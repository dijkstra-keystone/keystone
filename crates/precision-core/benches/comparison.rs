use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use precision_core::Decimal;
use rust_decimal::Decimal as RustDecimal;

fn addition_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("addition");

    let pk_a = Decimal::new(123456789, 6);
    let pk_b = Decimal::new(987654321, 6);

    let rd_a = RustDecimal::new(123456789, 6);
    let rd_b = RustDecimal::new(987654321, 6);

    group.bench_function("precision-core", |b| {
        b.iter(|| black_box(pk_a.checked_add(pk_b)))
    });

    group.bench_function("rust_decimal", |b| {
        b.iter(|| black_box(rd_a.checked_add(rd_b)))
    });

    group.finish();
}

fn subtraction_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("subtraction");

    let pk_a = Decimal::new(987654321, 6);
    let pk_b = Decimal::new(123456789, 6);

    let rd_a = RustDecimal::new(987654321, 6);
    let rd_b = RustDecimal::new(123456789, 6);

    group.bench_function("precision-core", |b| {
        b.iter(|| black_box(pk_a.checked_sub(pk_b)))
    });

    group.bench_function("rust_decimal", |b| {
        b.iter(|| black_box(rd_a.checked_sub(rd_b)))
    });

    group.finish();
}

fn multiplication_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiplication");

    let pk_a = Decimal::new(123456, 3);
    let pk_b = Decimal::new(789012, 3);

    let rd_a = RustDecimal::new(123456, 3);
    let rd_b = RustDecimal::new(789012, 3);

    group.bench_function("precision-core", |b| {
        b.iter(|| black_box(pk_a.checked_mul(pk_b)))
    });

    group.bench_function("rust_decimal", |b| {
        b.iter(|| black_box(rd_a.checked_mul(rd_b)))
    });

    group.finish();
}

fn division_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("division");

    let pk_a = Decimal::new(123456789, 6);
    let pk_b = Decimal::new(12345, 3);

    let rd_a = RustDecimal::new(123456789, 6);
    let rd_b = RustDecimal::new(12345, 3);

    group.bench_function("precision-core", |b| {
        b.iter(|| black_box(pk_a.checked_div(pk_b)))
    });

    group.bench_function("rust_decimal", |b| {
        b.iter(|| black_box(rd_a.checked_div(rd_b)))
    });

    group.finish();
}

fn mul_div_compound(c: &mut Criterion) {
    let mut group = c.benchmark_group("mul_div");

    let pk_a = Decimal::new(10000, 0);
    let pk_b = Decimal::new(80, 2);
    let pk_c = Decimal::new(5000, 0);

    let rd_a = RustDecimal::new(10000, 0);
    let rd_b = RustDecimal::new(80, 2);
    let rd_c = RustDecimal::new(5000, 0);

    group.bench_function("precision-core", |b| {
        b.iter(|| {
            let product = pk_a.checked_mul(pk_b).unwrap();
            black_box(product.checked_div(pk_c))
        })
    });

    group.bench_function("rust_decimal", |b| {
        b.iter(|| {
            let product = rd_a.checked_mul(rd_b).unwrap();
            black_box(product.checked_div(rd_c))
        })
    });

    group.finish();
}

fn compound_interest_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("compound_interest_12");

    let pk_rate = Decimal::ONE
        .checked_add(Decimal::new(5, 2))
        .unwrap();
    let rd_rate = RustDecimal::new(105, 2);

    group.bench_function("precision-core", |b| {
        b.iter(|| {
            let mut result = Decimal::ONE;
            for _ in 0..12 {
                result = result.checked_mul(pk_rate).unwrap();
            }
            black_box(result)
        })
    });

    group.bench_function("rust_decimal", |b| {
        b.iter(|| {
            let mut result = RustDecimal::ONE;
            for _ in 0..12 {
                result = result.checked_mul(rd_rate).unwrap();
            }
            black_box(result)
        })
    });

    group.finish();
}

fn health_factor_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("health_factor");

    let pk_collateral = Decimal::from(10_000i64);
    let pk_threshold = Decimal::new(80, 2);
    let pk_debt = Decimal::from(5_000i64);

    let rd_collateral = RustDecimal::new(10000, 0);
    let rd_threshold = RustDecimal::new(80, 2);
    let rd_debt = RustDecimal::new(5000, 0);

    group.bench_function("precision-core", |b| {
        b.iter(|| {
            let weighted = pk_collateral.checked_mul(pk_threshold).unwrap();
            black_box(weighted.checked_div(pk_debt))
        })
    });

    group.bench_function("rust_decimal", |b| {
        b.iter(|| {
            let weighted = rd_collateral.checked_mul(rd_threshold).unwrap();
            black_box(weighted.checked_div(rd_debt))
        })
    });

    group.finish();
}

fn swap_output_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("swap_output");

    let pk_r_in = Decimal::from(1_000_000i64);
    let pk_r_out = Decimal::from(1_000_000i64);
    let pk_amt = Decimal::from(10_000i64);
    let pk_fee = Decimal::new(997, 3);

    let rd_r_in = RustDecimal::new(1_000_000, 0);
    let rd_r_out = RustDecimal::new(1_000_000, 0);
    let rd_amt = RustDecimal::new(10_000, 0);
    let rd_fee = RustDecimal::new(997, 3);

    group.bench_function("precision-core", |b| {
        b.iter(|| {
            let effective = pk_amt.checked_mul(pk_fee).unwrap();
            let num = effective.checked_mul(pk_r_out).unwrap();
            let den = pk_r_in.checked_add(effective).unwrap();
            black_box(num.checked_div(den))
        })
    });

    group.bench_function("rust_decimal", |b| {
        b.iter(|| {
            let effective = rd_amt.checked_mul(rd_fee).unwrap();
            let num = effective.checked_mul(rd_r_out).unwrap();
            let den = rd_r_in.checked_add(effective).unwrap();
            black_box(num.checked_div(den))
        })
    });

    group.finish();
}

fn large_value_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_values");

    let pk_a = Decimal::from(999_999_999_999_999_999i64);
    let pk_b = Decimal::new(123456789012345, 8);

    let rd_a = RustDecimal::new(999_999_999_999_999_999, 0);
    let rd_b = RustDecimal::new(123456789012345, 8);

    group.bench_function("precision-core/mul", |b| {
        b.iter(|| black_box(pk_a.checked_mul(pk_b)))
    });

    group.bench_function("rust_decimal/mul", |b| {
        b.iter(|| black_box(rd_a.checked_mul(rd_b)))
    });

    group.bench_function("precision-core/div", |b| {
        b.iter(|| black_box(pk_a.checked_div(pk_b)))
    });

    group.bench_function("rust_decimal/div", |b| {
        b.iter(|| black_box(rd_a.checked_div(rd_b)))
    });

    group.finish();
}

criterion_group!(
    benches,
    addition_comparison,
    subtraction_comparison,
    multiplication_comparison,
    division_comparison,
    mul_div_compound,
    compound_interest_comparison,
    health_factor_comparison,
    swap_output_comparison,
    large_value_comparison,
);

criterion_main!(benches);
