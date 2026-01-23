//! 全面测试 - 测试所有电压值，包括小数点后两位

use battery_estimator::{BatteryChemistry, SocEstimator};

fn main() {
    println!("Battery SOC Estimator - Comprehensive Test");
    println!("==========================================\n");

    // 测试所有电池类型
    test_battery(BatteryChemistry::LiPo, "LiPo", 3.2, 4.2);
    test_battery(BatteryChemistry::LiFePO4, "LiFePO4", 2.5, 3.65);
    test_battery(BatteryChemistry::LiIon, "Li-Ion", 2.5, 4.2);
    test_battery(
        BatteryChemistry::Lipo410Full340Cutoff,
        "LiPo 4.1V/3.4V (Conservative)",
        3.4,
        4.1,
    );

    // 测试边界情况
    println!("\n\nBoundary Case Tests");
    println!("===================\n");
    test_boundary_cases();

    // 测试错误情况
    println!("\n\nError Case Tests");
    println!("================\n");
    test_error_cases();
}

/// 测试特定电池类型
fn test_battery(chemistry: BatteryChemistry, name: &str, min_v: f32, max_v: f32) {
    println!("Testing {} Battery ({}V - {}V):", name, min_v, max_v);
    println!("{}", "-".repeat(50));

    let estimator = SocEstimator::new(chemistry);

    // 1. 测试关键电压点
    println!("\nKey voltage points:");
    let key_voltages = generate_key_voltages(min_v, max_v);

    for voltage in key_voltages {
        match estimator.estimate_soc(voltage) {
            Ok(soc) => println!("  {:5.2}V -> {:6.2}%", voltage, soc),
            Err(e) => println!("  {:5.2}V -> ERROR: {}", voltage, e),
        }
    }

    // 2. 密集测试整个范围（步长0.01V）
    println!("\nDense testing (every 0.05V):");
    let mut voltage = min_v;
    let step = 0.05;

    while voltage <= max_v + 0.001 {
        // 添加微小容差
        match estimator.estimate_soc(voltage) {
            Ok(soc) => {
                // 只在SOC变化显著时打印
                if should_print(voltage, min_v, max_v, step) {
                    println!("  {:5.2}V -> {:6.2}%", voltage, soc);
                }
            }
            Err(e) => println!("  {:5.2}V -> ERROR: {}", voltage, e),
        }
        voltage += step;
    }

    // 3. 测试曲线特征点
    println!("\nCharacteristic points:");
    test_characteristic_points(&estimator, name);

    println!();
}

/// 生成关键电压点
fn generate_key_voltages(min_v: f32, max_v: f32) -> Vec<f32> {
    let mut voltages = Vec::new();

    // 边界点
    voltages.push(min_v);
    voltages.push(max_v);

    // 中间点
    let mid = (min_v + max_v) / 2.0;
    voltages.push(mid);

    // 四分位点
    voltages.push(min_v + (max_v - min_v) * 0.25);
    voltages.push(min_v + (max_v - min_v) * 0.75);

    // 特定测试点
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

/// 判断是否需要打印（减少输出量）
fn should_print(voltage: f32, min_v: f32, max_v: f32, step: f32) -> bool {
    // 总是打印边界和特定点
    if (voltage - min_v).abs() < step * 0.5 {
        return true;
    }
    if (voltage - max_v).abs() < step * 0.5 {
        return true;
    }

    // 打印每0.1V的点
    let tenth = (voltage * 10.0).round() / 10.0;
    (voltage - tenth).abs() < step * 0.5
}

/// 测试特征点
fn test_characteristic_points(estimator: &SocEstimator, name: &str) {
    match name {
        "LiPo" => {
            // LiPo 特征电压
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
            // LiFePO4 特征电压
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
            // Li-Ion 特征电压
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
            // LiPo 保守曲线特征电压
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

/// 测试边界情况
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

/// 测试错误情况
fn test_error_cases() {
    // 这里可以测试一些可能出错的情况
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
