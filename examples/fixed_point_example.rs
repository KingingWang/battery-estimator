//! Fixed-point arithmetic example
//!
//! This example demonstrates the high-performance fixed-point API for embedded systems
//! without hardware floating-point units (FPU).
//!
//! Run with:
//! ```bash
//! cargo run --example fixed_point_example --features fixed-point
//! ```

#[cfg(feature = "fixed-point")]
use battery_estimator::fixed_point::{Fixed, FixedBatteryChemistry, FixedSocEstimator};

#[cfg(feature = "fixed-point")]
fn main() {
    println!("=== Fixed-Point Battery SOC Estimation ===\n");

    // Create estimator for LiPo battery
    let estimator = FixedSocEstimator::new(FixedBatteryChemistry::LiPo);
    println!("Created fixed-point estimator for LiPo battery");

    // Get voltage range
    let (min_v, max_v) = estimator.voltage_range();
    println!(
        "Voltage range: {:.2}V to {:.2}V\n",
        min_v.to_num::<f32>(),
        max_v.to_num::<f32>()
    );

    // Test various voltages
    let test_voltages = [3.2, 3.4, 3.7, 4.0, 4.2];

    println!("SOC Estimation:");
    println!("Voltage | SOC");
    println!("--------|------");

    for &v in &test_voltages {
        let voltage = Fixed::from_num(v);
        match estimator.estimate_soc(voltage) {
            Ok(soc) => {
                let soc_f32 = soc.to_num::<f32>();
                println!("{:>6.2}V | {:>5.1}%", v, soc_f32);
            }
            Err(e) => println!("{:>6.2}V | Error: {}", v, e),
        }
    }

    println!("\n=== Temperature Compensation ===\n");

    // Create estimator with temperature compensation
    use battery_estimator::fixed_point::FixedEstimatorConfig;

    let config = FixedEstimatorConfig::default()
        .with_temperature_compensation()
        .with_nominal_temperature(Fixed::from_num(25.0))
        .with_temperature_coefficient(Fixed::from_num(0.005));

    let comp_estimator = FixedSocEstimator::with_config(FixedBatteryChemistry::LiPo, config);

    let test_voltage = Fixed::from_num(3.7);
    let temperatures = [0.0, 10.0, 25.0, 35.0, 50.0];

    println!("Voltage: 3.7V");
    println!("Temperature | Compensated SOC");
    println!("------------|----------------");

    for &temp in &temperatures {
        let temp_fixed = Fixed::from_num(temp);
        match comp_estimator.estimate_soc_compensated(test_voltage, temp_fixed) {
            Ok(soc) => {
                let soc_f32 = soc.to_num::<f32>();
                println!("{:>10.1}°C | {:>14.1}%", temp, soc_f32);
            }
            Err(e) => println!("{:>10.1}°C | Error: {}", temp, e),
        }
    }

    println!("\n=== All Battery Types ===\n");

    let battery_types = [
        (FixedBatteryChemistry::LiPo, "LiPo"),
        (FixedBatteryChemistry::LiFePO4, "LiFePO4"),
        (FixedBatteryChemistry::LiIon, "Li-Ion"),
        (
            FixedBatteryChemistry::Lipo410Full340Cutoff,
            "LiPo Conservative",
        ),
    ];

    let test_voltage = Fixed::from_num(3.7);

    println!("Voltage: 3.7V");
    println!("Battery Type        | SOC");
    println!("--------------------|------");

    for (chemistry, name) in &battery_types {
        let est = FixedSocEstimator::new(*chemistry);
        match est.estimate_soc(test_voltage) {
            Ok(soc) => {
                let soc_f32 = soc.to_num::<f32>();
                println!("{:<19} | {:>5.1}%", name, soc_f32);
            }
            Err(_) => {
                // Some battery types may not include 3.7V in their range
                println!("{:<19} | Out of range", name);
            }
        }
    }

    println!("\n=== Performance Notes ===");
    println!("✓ Fixed-point arithmetic is 2-10x faster on systems without FPU");
    println!("✓ Deterministic execution time");
    println!("✓ No floating-point library overhead");
    println!("✓ Precision: ~0.000015 (sufficient for battery estimation)");
}

#[cfg(not(feature = "fixed-point"))]
fn main() {
    eprintln!("This example requires the 'fixed-point' feature to be enabled.");
    eprintln!("Run with: cargo run --example fixed_point_example --features fixed-point");
    std::process::exit(1);
}
