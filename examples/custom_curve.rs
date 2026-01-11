//! 自定义曲线示例

use battery_estimator::{Curve, CurvePoint, SocEstimator};

fn main() {
    println!("Battery SOC Estimator - Custom Curve Example");
    println!("============================================\n");
    
    // 创建自定义曲线
    const CUSTOM_CURVE: Curve = Curve::new(&[
        CurvePoint::new(3.2, 0.0),
        CurvePoint::new(3.7, 50.0),
        CurvePoint::new(4.2, 100.0),
    ]);
    
    // 使用自定义曲线创建估算器
    let estimator = SocEstimator::with_custom_curve(&CUSTOM_CURVE);
    
    // 测试
    let voltages = [4.2, 4.0, 3.8, 3.7, 3.6, 3.5, 3.4, 3.2];
    
    println!("Custom curve:");
    for voltage in voltages.iter() {
        match estimator.estimate_soc(*voltage) {
            Ok(soc) => println!("  {:.1}V -> {:.1}%", voltage, soc),
            Err(e) => println!("  {:.1}V -> Error: {}", voltage, e),
        }
    }
}

