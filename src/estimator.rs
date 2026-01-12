//! SOC(荷电状态)估算器（集成温度补偿）

use crate::{
    BatteryChemistry, Curve, Error, compensate_aging, compensate_temperature, default_curves,
    default_temperature_compensation,
};

/// SOC估算器配置
#[derive(Debug, Clone, Copy)]
pub struct EstimatorConfig {
    /// 标称温度 (°C)
    pub nominal_temperature: f32,
    /// 温度补偿系数 (每°C变化百分比)
    pub temperature_coefficient: f32,
    /// 电池年龄 (年)
    pub age_years: f32,
    /// 老化系数 (每年容量损失百分比)
    pub aging_factor: f32,
    /// 是否启用温度补偿
    pub enable_temperature_compensation: bool,
    /// 是否启用老化补偿
    pub enable_aging_compensation: bool,
}

impl EstimatorConfig {
    /// 默认配置
    pub const fn default() -> Self {
        Self {
            nominal_temperature: 25.0,
            temperature_coefficient: 0.0005, // 0.05% 每°C
            age_years: 0.0,
            aging_factor: 0.02, // 每年2%容量损失
            enable_temperature_compensation: false,
            enable_aging_compensation: false,
        }
    }

    /// 启用温度补偿
    pub const fn with_temperature_compensation(mut self) -> Self {
        self.enable_temperature_compensation = true;
        self
    }

    /// 启用老化补偿
    pub const fn with_aging_compensation(mut self) -> Self {
        self.enable_aging_compensation = true;
        self
    }

    /// 设置标称温度
    pub const fn with_nominal_temperature(mut self, temp: f32) -> Self {
        self.nominal_temperature = temp;
        self
    }

    /// 设置温度系数
    pub const fn with_temperature_coefficient(mut self, coeff: f32) -> Self {
        self.temperature_coefficient = coeff;
        self
    }

    /// 设置电池年龄
    pub const fn with_age_years(mut self, years: f32) -> Self {
        self.age_years = years;
        self
    }

    /// 设置老化系数
    pub const fn with_aging_factor(mut self, factor: f32) -> Self {
        self.aging_factor = factor;
        self
    }
}

// 非const的Default实现
impl Default for EstimatorConfig {
    fn default() -> Self {
        // default 方法
        EstimatorConfig {
            nominal_temperature: 25.0,
            temperature_coefficient: 0.0005,
            age_years: 0.0,
            aging_factor: 0.02,
            enable_temperature_compensation: false,
            enable_aging_compensation: false,
        }
    }
}

/// SOC估算器
#[derive(Debug, Clone, Copy)]
pub struct SocEstimator {
    curve: &'static Curve,
    config: EstimatorConfig,
}

impl SocEstimator {
    /// 创建新的SOC估算器（默认配置）
    pub const fn new(chemistry: BatteryChemistry) -> Self {
        let curve = match chemistry {
            BatteryChemistry::LiPo => &default_curves::LIPO,
            BatteryChemistry::LiFePO4 => &default_curves::LIFEPO4,
            BatteryChemistry::LiIon => &default_curves::LIION,
            BatteryChemistry::Lipo410Full340Cutoff => &default_curves::LIPO410_FULL340_CUTOFF,
        };

        Self {
            curve,
            config: EstimatorConfig::default(), // 现在这是const函数
        }
    }

    /// 使用自定义曲线创建估算器
    pub const fn with_custom_curve(curve: &'static Curve) -> Self {
        Self {
            curve,
            config: EstimatorConfig::default(),
        }
    }

    /// 使用配置创建估算器（const版本）
    pub const fn with_config(chemistry: BatteryChemistry, config: EstimatorConfig) -> Self {
        let curve = match chemistry {
            BatteryChemistry::LiPo => &default_curves::LIPO,
            BatteryChemistry::LiFePO4 => &default_curves::LIFEPO4,
            BatteryChemistry::LiIon => &default_curves::LIION,
            BatteryChemistry::Lipo410Full340Cutoff => &default_curves::LIPO410_FULL340_CUTOFF,
        };

        Self { curve, config }
    }

    /// 使用配置创建估算器
    pub fn with_config_non_const(chemistry: BatteryChemistry, config: EstimatorConfig) -> Self {
        Self::with_config(chemistry, config)
    }

    /// 估算SOC（不带温度补偿）
    pub fn estimate_soc(&self, voltage: f32) -> Result<f32, Error> {
        self.curve.voltage_to_soc(voltage)
    }

    /// 估算SOC（带温度补偿） - 总是应用温度补偿，不考虑配置
    pub fn estimate_soc_with_temp(&self, voltage: f32, temperature: f32) -> Result<f32, Error> {
        let base_soc = self.curve.voltage_to_soc(voltage)?;

        // 总是应用温度补偿，使用默认参数
        let compensated = default_temperature_compensation(base_soc, temperature);

        Ok(compensated.clamp(0.0, 100.0))
    }

    /// 估算SOC（使用配置中的设置）
    pub fn estimate_soc_compensated(&self, voltage: f32, temperature: f32) -> Result<f32, Error> {
        let base_soc = self.curve.voltage_to_soc(voltage)?;
        let mut soc = base_soc;

        // 应用温度补偿
        if self.config.enable_temperature_compensation {
            soc = compensate_temperature(
                soc,
                temperature,
                self.config.nominal_temperature,
                self.config.temperature_coefficient,
            );
        }

        // 应用老化补偿
        if self.config.enable_aging_compensation {
            soc = compensate_aging(soc, self.config.age_years, self.config.aging_factor);
        }

        // 确保SOC在有效范围内
        Ok(soc.clamp(0.0, 100.0))
    }

    /// 获取电压范围
    pub const fn voltage_range(&self) -> (f32, f32) {
        self.curve.voltage_range()
    }

    /// 更新配置
    pub fn update_config(&mut self, config: EstimatorConfig) {
        self.config = config;
    }

    /// 获取当前配置
    pub const fn config(&self) -> &EstimatorConfig {
        &self.config
    }

    /// 启用温度补偿
    pub fn enable_temperature_compensation(&mut self, nominal_temp: f32, coefficient: f32) {
        self.config.enable_temperature_compensation = true;
        self.config.nominal_temperature = nominal_temp;
        self.config.temperature_coefficient = coefficient;
    }

    /// 启用老化补偿
    pub fn enable_aging_compensation(&mut self, age_years: f32, aging_factor: f32) {
        self.config.enable_aging_compensation = true;
        self.config.age_years = age_years;
        self.config.aging_factor = aging_factor;
    }

    /// 禁用所有补偿
    pub fn disable_all_compensation(&mut self) {
        self.config.enable_temperature_compensation = false;
        self.config.enable_aging_compensation = false;
    }
}

// 为简化使用添加一些便捷构造函数
impl SocEstimator {
    /// 创建带温度补偿的估算器
    pub fn with_temperature_compensation(
        chemistry: BatteryChemistry,
        nominal_temp: f32,
        coefficient: f32,
    ) -> Self {
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(nominal_temp)
            .with_temperature_coefficient(coefficient);

        Self::with_config_non_const(chemistry, config)
    }

    /// 创建带老化补偿的估算器
    pub fn with_aging_compensation(
        chemistry: BatteryChemistry,
        age_years: f32,
        aging_factor: f32,
    ) -> Self {
        let config = EstimatorConfig::default()
            .with_aging_compensation()
            .with_age_years(age_years)
            .with_aging_factor(aging_factor);

        Self::with_config_non_const(chemistry, config)
    }

    /// 创建带所有补偿的估算器
    pub fn with_all_compensation(
        chemistry: BatteryChemistry,
        nominal_temp: f32,
        temp_coeff: f32,
        age_years: f32,
        aging_factor: f32,
    ) -> Self {
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_aging_compensation()
            .with_nominal_temperature(nominal_temp)
            .with_temperature_coefficient(temp_coeff)
            .with_age_years(age_years)
            .with_aging_factor(aging_factor);

        Self::with_config_non_const(chemistry, config)
    }

    /// 使用默认温度补偿估算SOC
    pub fn estimate_soc_default_temp(&self, voltage: f32, temperature: f32) -> Result<f32, Error> {
        let base_soc = self.curve.voltage_to_soc(voltage)?;
        let compensated = default_temperature_compensation(base_soc, temperature);
        Ok(compensated.clamp(0.0, 100.0))
    }

    /// 使用默认温度补偿（简单接口）
    pub fn estimate_soc_simple(&self, voltage: f32, temperature: f32) -> Result<f32, Error> {
        self.estimate_soc_default_temp(voltage, temperature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimator_basic() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        // 测试边界
        assert!(estimator.estimate_soc(3.2).unwrap().abs() < 1.0);
        assert!(estimator.estimate_soc(4.2).unwrap() > 99.0);

        // 测试典型值
        let soc = estimator.estimate_soc(3.7).unwrap();
        assert!(
            soc >= 45.0 && soc <= 55.0,
            "3.7V should be around 50%, got {}",
            soc
        );
    }

    #[test]
    fn test_estimator_with_temp() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        // 测试不同温度
        let base_soc = estimator.estimate_soc(3.7).unwrap();
        let cold_soc = estimator.estimate_soc_with_temp(3.7, 0.0).unwrap();
        let hot_soc = estimator.estimate_soc_with_temp(3.7, 50.0).unwrap();

        // 低温应该显示更高的SOC
        assert!(cold_soc > base_soc, "Cold temp should increase SOC");
        // 高温应该显示更低的SOC
        assert!(hot_soc < base_soc, "Hot temp should decrease SOC");
    }

    #[test]
    fn test_estimator_custom_curve() {
        use crate::CurvePoint;
        const CUSTOM_CURVE: Curve = Curve::new(&[
            CurvePoint::new(3.0, 0.0),
            CurvePoint::new(3.5, 50.0),
            CurvePoint::new(4.0, 100.0),
        ]);

        let estimator = SocEstimator::with_custom_curve(&CUSTOM_CURVE);

        assert_eq!(estimator.estimate_soc(3.0).unwrap(), 0.0);
        assert_eq!(estimator.estimate_soc(3.5).unwrap(), 50.0);
        assert_eq!(estimator.estimate_soc(4.0).unwrap(), 100.0);
    }
}
