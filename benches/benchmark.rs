//! Performance benchmarks for battery-estimator
//!
//! Run with: cargo bench

use battery_estimator::{
    compensate_aging, compensate_aging_fixed, compensate_temperature, compensate_temperature_fixed,
    default_temperature_compensation, default_temperature_compensation_fixed, BatteryChemistry,
    Curve, CurvePoint, Fixed, SocEstimator,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// ============================================================================
// SOC Estimation Benchmarks
// ============================================================================

fn bench_estimate_soc(c: &mut Criterion) {
    let lipo = SocEstimator::new(BatteryChemistry::LiPo);
    let lifepo4 = SocEstimator::new(BatteryChemistry::LiFePO4);
    let liion = SocEstimator::new(BatteryChemistry::LiIon);
    let conservative = SocEstimator::new(BatteryChemistry::Lipo410Full340Cutoff);

    let mut group = c.benchmark_group("estimate_soc");

    group.bench_function("lipo_3_7v", |b| {
        b.iter(|| lipo.estimate_soc(black_box(3.7)))
    });

    group.bench_function("lifepo4_3_2v", |b| {
        b.iter(|| lifepo4.estimate_soc(black_box(3.2)))
    });

    group.bench_function("liion_3_7v", |b| {
        b.iter(|| liion.estimate_soc(black_box(3.7)))
    });

    group.bench_function("conservative_3_77v", |b| {
        b.iter(|| conservative.estimate_soc(black_box(3.77)))
    });

    group.finish();
}

fn bench_estimate_soc_fixed(c: &mut Criterion) {
    let estimator = SocEstimator::new(BatteryChemistry::LiPo);
    let voltage = Fixed::from_num(3.7);

    c.bench_function("estimate_soc_fixed", |b| {
        b.iter(|| estimator.estimate_soc_fixed(black_box(voltage)))
    });
}

fn bench_estimate_soc_with_temp(c: &mut Criterion) {
    let estimator = SocEstimator::new(BatteryChemistry::LiPo);

    let mut group = c.benchmark_group("estimate_soc_with_temp");

    group.bench_function("cold_temp", |b| {
        b.iter(|| estimator.estimate_soc_with_temp(black_box(3.7), black_box(0.0)))
    });

    group.bench_function("normal_temp", |b| {
        b.iter(|| estimator.estimate_soc_with_temp(black_box(3.7), black_box(25.0)))
    });

    group.bench_function("hot_temp", |b| {
        b.iter(|| estimator.estimate_soc_with_temp(black_box(3.7), black_box(50.0)))
    });

    group.finish();
}

fn bench_estimate_soc_compensated(c: &mut Criterion) {
    let estimator = SocEstimator::with_all_compensation(
        BatteryChemistry::LiPo,
        Fixed::from_num(25.0),
        Fixed::from_num(0.005),
        Fixed::from_num(2.0),
        Fixed::from_num(0.02),
    );

    c.bench_function("estimate_soc_compensated", |b| {
        b.iter(|| estimator.estimate_soc_compensated(black_box(3.7), black_box(20.0)))
    });
}

// ============================================================================
// Curve Benchmarks
// ============================================================================

fn bench_curve_operations(c: &mut Criterion) {
    let curve = Curve::new(&[
        CurvePoint::new(3.0, 0.0),
        CurvePoint::new(3.5, 50.0),
        CurvePoint::new(4.0, 100.0),
    ]);

    let mut group = c.benchmark_group("curve");

    group.bench_function("voltage_to_soc", |b| {
        b.iter(|| curve.voltage_to_soc(black_box(3.5)))
    });

    group.bench_function("voltage_to_soc_fixed", |b| {
        b.iter(|| curve.voltage_to_soc_fixed(black_box(Fixed::from_num(3.5))))
    });

    group.bench_function("voltage_range", |b| b.iter(|| curve.voltage_range()));

    group.finish();
}

fn bench_curve_creation(c: &mut Criterion) {
    c.bench_function("curve_new_small", |b| {
        b.iter(|| Curve::new(&[CurvePoint::new(3.0, 0.0), CurvePoint::new(4.0, 100.0)]))
    });

    c.bench_function("curve_new_large", |b| {
        b.iter(|| {
            Curve::new(&[
                CurvePoint::new(3.0, 0.0),
                CurvePoint::new(3.1, 10.0),
                CurvePoint::new(3.2, 20.0),
                CurvePoint::new(3.3, 30.0),
                CurvePoint::new(3.4, 40.0),
                CurvePoint::new(3.5, 50.0),
                CurvePoint::new(3.6, 60.0),
                CurvePoint::new(3.7, 70.0),
                CurvePoint::new(3.8, 80.0),
                CurvePoint::new(3.9, 90.0),
                CurvePoint::new(4.0, 100.0),
            ])
        })
    });
}

// ============================================================================
// Compensation Benchmarks
// ============================================================================

fn bench_temperature_compensation(c: &mut Criterion) {
    let mut group = c.benchmark_group("temperature_compensation");

    group.bench_function("default_f32", |b| {
        b.iter(|| default_temperature_compensation(black_box(50.0), black_box(0.0)))
    });

    group.bench_function("default_fixed", |b| {
        b.iter(|| {
            default_temperature_compensation_fixed(
                black_box(Fixed::from_num(50.0)),
                black_box(Fixed::from_num(0.0)),
            )
        })
    });

    group.bench_function("custom_f32", |b| {
        b.iter(|| {
            compensate_temperature(
                black_box(50.0),
                black_box(0.0),
                black_box(25.0),
                black_box(0.005),
            )
        })
    });

    group.bench_function("custom_fixed", |b| {
        b.iter(|| {
            compensate_temperature_fixed(
                black_box(Fixed::from_num(50.0)),
                black_box(Fixed::from_num(0.0)),
                black_box(Fixed::from_num(25.0)),
                black_box(Fixed::from_num(0.005)),
            )
        })
    });

    group.finish();
}

fn bench_aging_compensation(c: &mut Criterion) {
    let mut group = c.benchmark_group("aging_compensation");

    group.bench_function("f32", |b| {
        b.iter(|| compensate_aging(black_box(50.0), black_box(2.0), black_box(0.02)))
    });

    group.bench_function("fixed", |b| {
        b.iter(|| {
            compensate_aging_fixed(
                black_box(Fixed::from_num(50.0)),
                black_box(Fixed::from_num(2.0)),
                black_box(Fixed::from_num(0.02)),
            )
        })
    });

    group.finish();
}

// ============================================================================
// CurvePoint Benchmarks
// ============================================================================

fn bench_curve_point(c: &mut Criterion) {
    let mut group = c.benchmark_group("curve_point");

    group.bench_function("new", |b| {
        b.iter(|| CurvePoint::new(black_box(3.7), black_box(50.0)))
    });

    group.bench_function("from_fixed", |b| {
        b.iter(|| {
            CurvePoint::from_fixed(
                black_box(Fixed::from_num(3.7)),
                black_box(Fixed::from_num(50.0)),
            )
        })
    });

    group.bench_function("voltage", |b| {
        let point = CurvePoint::new(3.7, 50.0);
        b.iter(|| point.voltage())
    });

    group.bench_function("soc", |b| {
        let point = CurvePoint::new(3.7, 50.0);
        b.iter(|| point.soc())
    });

    group.finish();
}

// ============================================================================
// Estimator Creation Benchmarks
// ============================================================================

fn bench_estimator_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("estimator_creation");

    group.bench_function("new_lipo", |b| {
        b.iter(|| SocEstimator::new(BatteryChemistry::LiPo))
    });

    group.bench_function("new_lifepo4", |b| {
        b.iter(|| SocEstimator::new(BatteryChemistry::LiFePO4))
    });

    group.bench_function("with_temperature_compensation", |b| {
        b.iter(|| {
            SocEstimator::with_temperature_compensation(
                BatteryChemistry::LiPo,
                Fixed::from_num(25.0),
                Fixed::from_num(0.005),
            )
        })
    });

    group.bench_function("with_aging_compensation", |b| {
        b.iter(|| {
            SocEstimator::with_aging_compensation(
                BatteryChemistry::LiPo,
                Fixed::from_num(2.0),
                Fixed::from_num(0.02),
            )
        })
    });

    group.bench_function("with_all_compensation", |b| {
        b.iter(|| {
            SocEstimator::with_all_compensation(
                BatteryChemistry::LiPo,
                Fixed::from_num(25.0),
                Fixed::from_num(0.005),
                Fixed::from_num(2.0),
                Fixed::from_num(0.02),
            )
        })
    });

    group.finish();
}

// ============================================================================
// Custom Curve Estimator
// ============================================================================

fn bench_custom_curve(c: &mut Criterion) {
    const CUSTOM_CURVE: Curve = Curve::new(&[
        CurvePoint::new(3.0, 0.0),
        CurvePoint::new(3.5, 50.0),
        CurvePoint::new(4.0, 100.0),
    ]);

    let estimator = SocEstimator::with_custom_curve(&CUSTOM_CURVE);

    c.bench_function("custom_curve_estimate", |b| {
        b.iter(|| estimator.estimate_soc(black_box(3.5)))
    });
}

// ============================================================================
// Boundary Cases
// ============================================================================

fn bench_boundary_cases(c: &mut Criterion) {
    let estimator = SocEstimator::new(BatteryChemistry::LiPo);

    let mut group = c.benchmark_group("boundary_cases");

    // Min voltage
    group.bench_function("min_voltage", |b| {
        b.iter(|| estimator.estimate_soc(black_box(3.2)))
    });

    // Max voltage
    group.bench_function("max_voltage", |b| {
        b.iter(|| estimator.estimate_soc(black_box(4.2)))
    });

    // Below min (should return min SOC)
    group.bench_function("below_min", |b| {
        b.iter(|| estimator.estimate_soc(black_box(3.0)))
    });

    // Above max (should return max SOC)
    group.bench_function("above_max", |b| {
        b.iter(|| estimator.estimate_soc(black_box(4.5)))
    });

    group.finish();
}

// ============================================================================
// Throughput Test - Multiple Voltages
// ============================================================================

fn bench_throughput(c: &mut Criterion) {
    let estimator = SocEstimator::new(BatteryChemistry::LiPo);
    let voltages: [f32; 100] = core::array::from_fn(|i| {
        3.2 + (i as f32 * 0.01) // 3.20 to 4.19
    });

    c.bench_function("throughput_100_estimations", |b| {
        b.iter(|| {
            for v in &voltages {
                let _ = estimator.estimate_soc(black_box(*v));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_estimate_soc,
    bench_estimate_soc_fixed,
    bench_estimate_soc_with_temp,
    bench_estimate_soc_compensated,
    bench_curve_operations,
    bench_curve_creation,
    bench_temperature_compensation,
    bench_aging_compensation,
    bench_curve_point,
    bench_estimator_creation,
    bench_custom_curve,
    bench_boundary_cases,
    bench_throughput,
);

criterion_main!(benches);
