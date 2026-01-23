//! Temperature Compensation Test
//!
//! Tests SOC changes at different temperatures, demonstrating temperature compensation effects

use battery_estimator::{default_temperature_compensation, BatteryChemistry, SocEstimator};

fn main() {
    println!("Battery SOC Estimator - Temperature Compensation Test");
    println!("=====================================================\n");

    // Test all battery types at different temperatures
    test_all_batteries_with_temperature();

    // Detailed temperature compensation demonstration
    println!("\n\nDetailed Temperature Compensation Analysis");
    println!("==========================================\n");
    detailed_temperature_analysis();

    // Test extreme temperature conditions
    println!("\n\nExtreme Temperature Tests");
    println!("========================\n");
    test_extreme_temperatures();
}

/// Test all battery types at different temperatures
fn test_all_batteries_with_temperature() {
    let chemistries = [
        (BatteryChemistry::LiPo, "LiPo", 3.2, 4.2),
        (BatteryChemistry::LiFePO4, "LiFePO4", 2.5, 3.65),
        (BatteryChemistry::LiIon, "Li-Ion", 2.5, 4.2),
        (
            BatteryChemistry::Lipo410Full340Cutoff,
            "LiPo 4.1V/3.4V (Conservative)",
            3.4,
            4.1,
        ),
    ];

    for (chem, name, min_v, max_v) in chemistries.iter() {
        println!("Testing {} Battery Temperature Effects:", name);
        println!("{}", "-".repeat(60));

        let estimator = SocEstimator::new(*chem);

        // Test key voltage points at different temperatures
        let test_temperatures = [-10.0, 0.0, 25.0, 40.0, 60.0];
        let test_voltages = generate_test_voltages(*min_v, *max_v);

        println!("\nVoltage | {:^48}", "SOC at Different Temperatures (°C)");
        println!("--------|{}", "-".repeat(50));
        print!("        |");
        for temp in test_temperatures.iter() {
            print!(" {:>7.0}°C |", temp);
        }
        println!();
        println!("--------|{}", "-".repeat(50));

        for voltage in test_voltages.iter() {
            match estimator.estimate_soc(*voltage) {
                Ok(base_soc) => {
                    print!(" {:6.2}V |", voltage);

                    for temp in test_temperatures.iter() {
                        let compensated = default_temperature_compensation(base_soc, *temp);
                        print!(" {:8.2}% |", compensated);
                    }
                    println!();
                }
                Err(e) => println!(" {:6.2}V | ERROR: {}", voltage, e),
            }
        }

        println!();

        // Show temperature impact on SOC percentage
        show_temperature_impact(*chem, *min_v, *max_v);
        println!();
    }
}

/// Generate test voltage points
fn generate_test_voltages(min_v: f32, max_v: f32) -> Vec<f32> {
    let mut voltages = Vec::new();

    // Boundary points
    voltages.push(min_v);
    voltages.push(max_v);

    // Midpoints (25%, 50%, 75%)
    let range = max_v - min_v;
    voltages.push(min_v + range * 0.25);
    voltages.push(min_v + range * 0.50);
    voltages.push(min_v + range * 0.75);

    // Specific test points (every 0.1V)
    let step = 0.1;
    let mut v = min_v + step;
    while v < max_v {
        voltages.push(v);
        v += step;
    }

    voltages.sort_by(|a, b| a.partial_cmp(b).unwrap());
    voltages.dedup();
    voltages
}

/// Show temperature impact on SOC percentage
fn show_temperature_impact(chemistry: BatteryChemistry, min_v: f32, max_v: f32) {
    let estimator = SocEstimator::new(chemistry);
    let mid_voltage = (min_v + max_v) / 2.0;

    match estimator.estimate_soc(mid_voltage) {
        Ok(base_soc) => {
            println!(
                "Temperature impact at {:.2}V (base SOC: {:.1}%):",
                mid_voltage, base_soc
            );

            let temperatures = [-20.0, -10.0, 0.0, 10.0, 20.0, 25.0, 30.0, 40.0, 50.0, 60.0];

            println!("  Temp (°C) | SOC (%) | Change (%) | Effect");
            println!("  ----------|---------|------------|--------");

            for temp in temperatures.iter() {
                let compensated = default_temperature_compensation(base_soc, *temp);
                let change = compensated - base_soc;
                let change_percent = (change / base_soc) * 100.0;

                let effect = if change.abs() < 0.1 {
                    "Negligible"
                } else if change > 0.0 {
                    "Increased"
                } else {
                    "Decreased"
                };

                println!(
                    "  {:>9.0} | {:7.1} | {:>+10.2} | {}",
                    temp, compensated, change_percent, effect
                );
            }
        }
        Err(e) => println!("Error calculating temperature impact: {}", e),
    }
}

/// Detailed temperature compensation analysis
fn detailed_temperature_analysis() {
    let estimator = SocEstimator::new(BatteryChemistry::LiPo);

    // Test voltage changes at different temperatures
    let test_voltages = [3.2, 3.5, 3.7, 3.9, 4.2];
    let test_temperatures = [-20.0, 0.0, 25.0, 50.0];

    println!("Detailed analysis of temperature compensation:");
    println!("(Showing base SOC and compensated SOC at different temperatures)");
    println!();

    for voltage in test_voltages.iter() {
        match estimator.estimate_soc(*voltage) {
            Ok(base_soc) => {
                println!("At {:.1}V (base SOC: {:.1}%):", voltage, base_soc);

                for temp in test_temperatures.iter() {
                    let compensated = default_temperature_compensation(base_soc, *temp);
                    let diff = compensated - base_soc;

                    println!(
                        "  {:4.0}°C: {:.1}% ({:+.1}%, {:.1}% relative change)",
                        temp,
                        compensated,
                        diff,
                        (diff / base_soc) * 100.0
                    );
                }
                println!();
            }
            Err(e) => println!("Error at {:.1}V: {}", voltage, e),
        }
    }

    // Show temperature coefficient effects
    println!("Temperature coefficient explanation:");
    println!("  Default coefficient: 0.0005 (0.05% per °C)");
    println!("  This means for every °C away from 25°C:");
    println!("    SOC changes by 0.05% of its current value");
    println!();

    let example_soc = 50.0;
    let example_temps = [0.0, 50.0];

    for temp in example_temps.iter() {
        let delta_temp = temp - 25.0;
        let compensation: f32 = delta_temp * 0.0005 * 100.0; // Convert to percentage
        let bounded_compensation = compensation.clamp(-5.0, 5.0);
        let final_soc = example_soc * (1.0 - bounded_compensation / 100.0);

        println!("  Example at {}°C (Δ={}°C from 25°C):", temp, delta_temp);
        println!("    Theoretical compensation: {:.2}%", compensation);
        println!(
            "    Bounded compensation: {:.2}% (max ±5%)",
            bounded_compensation
        );
        println!("    Final SOC: {:.1}% (from 50.0%)", final_soc);
        println!();
    }
}

/// Test extreme temperature conditions
fn test_extreme_temperatures() {
    let estimator = SocEstimator::new(BatteryChemistry::LiPo);
    let voltage = 3.7;

    match estimator.estimate_soc(voltage) {
        Ok(base_soc) => {
            println!(
                "Testing extreme temperatures at {:.1}V (base SOC: {:.1}%):",
                voltage, base_soc
            );

            let extreme_temps = [
                (-40.0, "Extreme cold (arctic winter)"),
                (-20.0, "Very cold (winter)"),
                (0.0, "Freezing"),
                (25.0, "Room temperature"),
                (40.0, "Hot day"),
                (60.0, "Very hot"),
                (85.0, "Maximum operating temp"),
            ];

            println!("  Temperature        | Description              | SOC (%) | Change (%)");
            println!("  -------------------|--------------------------|---------|-----------");

            for (temp, desc) in extreme_temps.iter() {
                let compensated = default_temperature_compensation(base_soc, *temp);
                let change = compensated - base_soc;

                println!(
                    "  {:>7.0}°C         | {:24} | {:7.1} | {:>+9.1}",
                    temp, desc, compensated, change
                );
            }

            println!();
            println!("Key observations:");
            println!("  1. Temperature compensation is bounded to ±5% maximum");
            println!("  2. At extreme cold (-40°C): SOC appears higher (battery less efficient)");
            println!("  3. At extreme heat (85°C): SOC appears lower (battery ages faster)");
            println!("  4. This is a simplified model - real batteries have more complex behavior");
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test compensation boundaries
    println!("\nTesting compensation bounds:");
    test_compensation_bounds();
}

/// Test compensation boundaries
fn test_compensation_bounds() {
    let estimator = SocEstimator::new(BatteryChemistry::LiPo);
    let voltage = 3.7;

    match estimator.estimate_soc(voltage) {
        Ok(base_soc) => {
            // Test cases exceeding ±5% compensation limit
            let test_cases = [
                (-100.0, "Should be limited to +5%"),
                (-50.0, "Should be limited to +5%"),
                (100.0, "Should be limited to -5%"),
                (150.0, "Should be limited to -5%"),
            ];

            for (temp, desc) in test_cases.iter() {
                let compensated = default_temperature_compensation(base_soc, *temp);
                let change = compensated - base_soc;
                let change_percent = (change / base_soc) * 100.0;

                println!(
                    "  {:>6.0}°C: {} -> Change: {:.1}% ({:.1}%)",
                    temp, desc, change, change_percent
                );
            }

            println!();
            println!(
                "Note: Temperature compensation is clamped to ±5% to prevent unrealistic values."
            );
        }
        Err(e) => println!("Error: {}", e),
    }
}
