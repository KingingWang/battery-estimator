//! Comprehensive Test - Testing all voltage values including two decimal places

use battery_estimator::{BatteryChemistry, SocEstimator};

fn main() {
    println!("Battery SOC Estimator - Comprehensive Test");
    println!("==========================================\n");

    // Test all battery types
    test_battery(BatteryChemistry::LiPo, "LiPo", 3.2, 4.2);
    test_battery(BatteryChemistry::LiFePO4, "LiFePO4", 2.5, 3.65);
    test_battery(BatteryChemistry::LiIon, "Li-Ion", 2.5, 4.2);
    test_battery(
        BatteryChemistry::Lipo410Full340Cutoff,
        "LiPo 4.1V/3.4V (Conservative)",
        3.4,
        4.1,
    );

    // Test boundary cases
    println!("\n\nBoundary Case Tests");
    println!("===================\n");
    test_boundary_cases();

    // Test error cases
    println!("\n\nError Case Tests");
    println!("================\n");
    test_error_cases();
}

/// Test specific battery type
fn test_battery(chemistry: BatteryChemistry, name: &str, min_v: f32, max_v: f32) {
    println!("Testing {} Battery ({}V - {}V):", name, min_v, max_v);
    println!("{}", "-".repeat(50));

    let estimator = SocEstimator::new(chemistry);

    // 1. Test key voltage points
    println!("\nKey voltage points:");
    let key_voltages = generate_key_voltages(min_v, max_v);

    for voltage in key_voltages {
        match estimator.estimate_soc(voltage) {
            Ok(soc) => println!("  {:5.2}V -> {:6.2}%", voltage, soc),
            Err(e) => println!("  {:5.2}V -> ERROR: {}", voltage, e),
        }
    }

    // 2. Dense testing of entire range (step 0.01V)
    println!("\nDense testing (every 0.05V):");
    let mut voltage = min_v;
    let step = 0.05;

    while voltage <= max_v + 0.001 {
        // Add small tolerance
        match estimator.estimate_soc(voltage) {
            Ok(soc) => {
                // Only print when SOC changes significantly
                if should_print(voltage, min_v, max_v, step) {
                    println!("  {:5.2}V -> {:6.2}%", voltage, soc);
                }
            }
            Err(e) => println!("  {:5.2}V -> ERROR: {}", voltage, e),
        }
        voltage += step;
    }

    // 3. Test curve characteristic points
    println!("\nCharacteristic points:");
    test_characteristic_points(&estimator, name);

    println!();
}

/// Generate key voltage points
fn generate_key_voltages(min_v: f32, max_v: f32) -> Vec<f32> {
    let mut voltages = Vec::new();

    // Boundary points
    voltages.push(min_v);
    voltages.push(max_v);

    // Midpoint
    let mid = (min_v + max_v) / 2.0;
    voltages.push(mid);

    // Quartile points
    voltages.push(min_v + (max_v - min_v) * 0.25);
    voltages.push(min_v + (max_v - min_v) * 0.75);

    // Specific test points
    let step = 0.1;
    let mut v = min_v;
    while v <= max_v + 0.001 {
        voltages.push(v);
        v += step;
    }

    voltages.sort_by(|a, b| a.partial_cmp(b).unwrap());
    voltages.dedup();
    voltages
}

/// Determine whether printing is needed
fn should_print(voltage: f32, min_v: f32, max_v: f32, step: f32) -> bool {
    // Always print boundaries and specific points
    if (voltage - min_v).abs() < step * 0.5 {
        return true;
    }
    if (voltage - max_v).abs() < step * 0.5 {
        return true;
    }

    // Print every 0.1V point
    let tenth = (voltage * 10.0).round() / 10.0;
    (voltage - tenth).abs() < step * 0.5
}

/// Test characteristic points
fn test_characteristic_points(estimator: &SocEstimator, name: &str) {
    match name {
        "LiPo" => {
            // LiPo characteristic voltages
            let points = [
                (3.2, "Discharge cutoff"),
                (3.7, "Nominal voltage"),
                (3.8, "Mid discharge"),
                (3.9, "High discharge"),
                (4.0, "Near full"),
                (4.2, "Full charge"),
            ];

            for (voltage, desc) in points.iter() {
                match estimator.estimate_soc(*voltage) {
                    Ok(soc) => println!("  {:5.2}V ({:20}) -> {:6.2}%", voltage, desc, soc),
                    Err(e) => println!("  {:5.2}V ({:20}) -> ERROR: {}", voltage, desc, e),
                }
            }
        }
        "LiFePO4" => {
            // LiFePO4 characteristic voltages
            let points = [
                (2.5, "Discharge cutoff"),
                (3.0, "Low voltage"),
                (3.2, "Nominal voltage"),
                (3.3, "Flat region"),
                (3.4, "High voltage"),
                (3.65, "Full charge"),
            ];

            for (voltage, desc) in points.iter() {
                match estimator.estimate_soc(*voltage) {
                    Ok(soc) => println!("  {:5.2}V ({:20}) -> {:6.2}%", voltage, desc, soc),
                    Err(e) => println!("  {:5.2}V ({:20}) -> ERROR: {}", voltage, desc, e),
                }
            }
        }
        "Li-Ion" => {
            // Li-Ion characteristic voltages
            let points = [
                (2.5, "Discharge cutoff"),
                (3.0, "Low voltage"),
                (3.7, "Nominal voltage"),
                (4.0, "High voltage"),
                (4.2, "Full charge"),
            ];

            for (voltage, desc) in points.iter() {
                match estimator.estimate_soc(*voltage) {
                    Ok(soc) => println!("  {:5.2}V ({:20}) -> {:6.2}%", voltage, desc, soc),
                    Err(e) => println!("  {:5.2}V ({:20}) -> ERROR: {}", voltage, desc, e),
                }
            }
        }
        "LiPo 4.1V/3.4V (Conservative)" => {
            // LiPo conservative curve characteristic voltages
            let points = [
                (3.4, "Shutdown cutoff (0%)"),
                (3.5, "Very low (10%)"),
                (3.7, "Low voltage (40%)"),
                (3.77, "Mid voltage (50%)"),
                (3.9, "High voltage (80%)"),
                (4.03, "Near full (95%)"),
                (4.1, "Full charge (100%)"),
            ];

            for (voltage, desc) in points.iter() {
                match estimator.estimate_soc(*voltage) {
                    Ok(soc) => println!("  {:5.2}V ({:20}) -> {:6.2}%", voltage, desc, soc),
                    Err(e) => println!("  {:5.2}V ({:20}) -> ERROR: {}", voltage, desc, e),
                }
            }
        }
        _ => {}
    }
}

/// Test boundary cases
fn test_boundary_cases() {
    let lipo = SocEstimator::new(BatteryChemistry::LiPo);

    println!("Exact boundary values:");
    println!("  3.20V -> {:.2}%", lipo.estimate_soc(3.20).unwrap());
    println!("  4.20V -> {:.2}%", lipo.estimate_soc(4.20).unwrap());

    println!("\nJust outside boundaries:");
    println!("  3.19V -> {:?}", lipo.estimate_soc(3.19));
    println!("  4.21V -> {:?}", lipo.estimate_soc(4.21));

    println!("\nVery close to boundaries:");
    println!("  3.2001V -> {:.2}%", lipo.estimate_soc(3.2001).unwrap());
    println!("  4.1999V -> {:.2}%", lipo.estimate_soc(4.1999).unwrap());

    println!("\nMidpoint exactly:");
    let midpoint = (3.2 + 4.2) / 2.0;
    println!(
        "  {:.4}V -> {:.2}%",
        midpoint,
        lipo.estimate_soc(midpoint).unwrap()
    );
}

/// Test error cases
fn test_error_cases() {
    // Test potential error scenarios here
    println!("Testing with extreme values:");

    let lipo = SocEstimator::new(BatteryChemistry::LiPo);

    let extreme_voltages = [0.0, -1.0, 10.0, f32::NAN, f32::INFINITY];

    for voltage in extreme_voltages.iter() {
        match lipo.estimate_soc(*voltage) {
            Ok(soc) => println!("  {:?}V -> {:.2}%", voltage, soc),
            Err(e) => println!("  {:?}V -> ERROR: {}", voltage, e),
        }
    }
}
