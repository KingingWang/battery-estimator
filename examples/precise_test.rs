//! 精确测试 - 测试每个0.01V间隔

use battery_estimator::{BatteryChemistry, SocEstimator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Battery SOC Estimator - Precise Voltage Test");
    println!("============================================\n");

    // 测试所有电池类型
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
        let step = 0.01; // 0.01V 精度

        // 生成测试电压
        let test_count = ((max_v - min_v) / step) as usize + 1;
        println!("  Voltage range: {}V to {}V", min_v, max_v);
        println!("  Step: {}V", step);
        println!("  Total test points: {}", test_count);

        // 执行测试
        let mut voltage = *min_v;
        let mut last_soc = -1.0;
        let mut significant_changes = 0;

        while voltage <= *max_v + 0.0005 {
            // 添加微小容差
            match estimator.estimate_soc(voltage) {
                Ok(soc) => {
                    // 只在SOC变化超过0.1%时打印
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
            // 处理浮点数精度
            voltage = (voltage * 100.0).round() / 100.0;
        }

        println!("  Significant SOC changes: {}", significant_changes);
        println!();
    }

    println!("\nGenerating summary...");

    Ok(())
}
