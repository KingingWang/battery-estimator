//! Precise Test - Testing at 0.01V Intervals

use battery_estimator::{BatteryChemistry, SocEstimator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Battery SOC Estimator - Precise Voltage Test");
    println!("============================================\n");

    // Test all battery types
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
        println!("Testing {} Battery:", name);

        let estimator = SocEstimator::new(*chem);
        let step = 0.01; // 0.01V precision

        // Generate test voltages
        let test_count = ((max_v - min_v) / step) as usize + 1;
        println!("  Voltage range: {}V to {}V", min_v, max_v);
        println!("  Step: {}V", step);
        println!("  Total test points: {}", test_count);

        // Execute test
        let mut voltage = *min_v;
        let mut last_soc = -1.0;
        let mut significant_changes = 0;

        while voltage <= *max_v + 0.0005 {
            // Add small tolerance
            match estimator.estimate_soc(voltage) {
                Ok(soc) => {
                    // Only print when SOC changes by more than 0.1%
                    if (soc - last_soc).abs() > 0.1 || voltage == *min_v || voltage == *max_v {
                        println!("    {:5.2}V -> {:6.2}%", voltage, soc);
                        last_soc = soc;
                        significant_changes += 1;
                    }
                }
                Err(e) => {
                    println!("    {:5.2}V -> ERROR: {}", voltage, e);
                }
            }

            voltage += step;
            // Handle floating-point precision
            voltage = (voltage * 100.0).round() / 100.0;
        }

        println!("  Significant SOC changes: {}", significant_changes);
        println!();
    }

    println!("\nGenerating summary...");

    Ok(())
}
