//! 温度补偿和老化补偿

/// 温度补偿
///
/// # Arguments
/// * `soc` - 基础SOC百分比
/// * `temperature` - 当前温度 (°C)
/// * `nominal_temp` - 标称温度 (°C)
/// * `coefficient` - 温度系数 (每°C变化百分比)
///
/// # 返回
/// 补偿后的SOC百分比
pub fn compensate_temperature(
    soc: f32,
    temperature: f32,
    nominal_temp: f32,
    coefficient: f32,
) -> f32 {
    let delta_temp = temperature - nominal_temp;
    let compensation = delta_temp * coefficient;

    // 限制补偿范围在 ±5% 以内
    let bounded_compensation = clamp(compensation, -0.05, 0.05);

    soc * (1.0 - bounded_compensation)
}

/// 老化补偿
///
/// # Arguments
/// * `soc` - 基础SOC百分比
/// * `age_years` - 电池年龄 (年)
/// * `aging_factor` - 老化系数 (每年容量损失百分比)
///
/// # 返回
/// 补偿后的SOC百分比
pub fn compensate_aging(soc: f32, age_years: f32, aging_factor: f32) -> f32 {
    let age_compensation = age_years * aging_factor;
    soc * (1.0 - clamp(age_compensation, 0.0, 0.5)) // 最大补偿50%
}

/// 通用温度补偿（使用默认系数）
pub fn default_temperature_compensation(soc: f32, temperature: f32) -> f32 {
    const NOMINAL_TEMP: f32 = 25.0;
    const COEFFICIENT: f32 = 0.0005; // 0.05% 每°C

    compensate_temperature(soc, temperature, NOMINAL_TEMP, COEFFICIENT)
}

/// 限制值在范围内
fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_compensation() {
        // 室温应该无变化
        assert_eq!(default_temperature_compensation(50.0, 25.0), 50.0);

        // 低温应该增加SOC
        let cold_compensated = default_temperature_compensation(50.0, 0.0);
        assert!(cold_compensated > 50.0, "Cold should increase SOC");

        // 高温应该减少SOC
        let hot_compensated = default_temperature_compensation(50.0, 50.0);
        assert!(hot_compensated < 50.0, "Hot should decrease SOC");
    }

    #[test]
    fn test_temperature_compensation_bounds() {
        // 测试边界限制（±5%）
        let extreme_cold = default_temperature_compensation(50.0, -100.0);
        let extreme_hot = default_temperature_compensation(50.0, 150.0);

        // 应该被限制在±5%以内
        assert!(extreme_cold <= 50.0 * 1.05);
        assert!(extreme_hot >= 50.0 * 0.95);
    }

    #[test]
    fn test_aging_compensation() {
        // 新电池应该无变化
        assert_eq!(compensate_aging(50.0, 0.0, 0.02), 50.0);

        // 老化电池应该减少SOC
        let aged = compensate_aging(50.0, 5.0, 0.02);
        assert!(aged < 50.0, "Aging should decrease SOC");

        // 测试最大补偿50%
        let very_aged = compensate_aging(50.0, 30.0, 0.02);
        assert!(
            very_aged >= 25.0,
            "Should be limited to 50% max compensation"
        );
    }
}
