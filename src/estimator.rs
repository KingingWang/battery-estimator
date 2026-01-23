//! SOC (State of Charge) Estimator with Temperature Compensation

use crate::{
    compensate_aging, compensate_temperature, default_curves, default_temperature_compensation,
    BatteryChemistry, Curve, Error,
};

/// SOC estimator configuration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EstimatorConfig {
    /// Nominal temperature (°C)
    pub nominal_temperature: f32,
    /// Temperature compensation coefficient (percentage change per °C)
    pub temperature_coefficient: f32,
    /// Battery age (years)
    pub age_years: f32,
    /// Aging factor (capacity loss percentage per year)
    pub aging_factor: f32,
    /// Compensation flags (bit field compression)
    flags: u8,
}

impl EstimatorConfig {
    /// Default configuration
    #[inline]
    pub const fn default() -> Self {
        Self {
            nominal_temperature: 25.0,
            temperature_coefficient: 0.0005, // 0.05% per °C
            age_years: 0.0,
            aging_factor: 0.02, // 2% capacity loss per year
            flags: 0,
        }
    }

    /// Enable temperature compensation
    #[inline]
    pub const fn with_temperature_compensation(mut self) -> Self {
        self.flags |= 0x01;
        self
    }

    /// Enable aging compensation
    #[inline]
    pub const fn with_aging_compensation(mut self) -> Self {
        self.flags |= 0x02;
        self
    }

    /// Set nominal temperature
    #[inline]
    pub const fn with_nominal_temperature(mut self, temp: f32) -> Self {
        self.nominal_temperature = temp;
        self
    }

    /// Set temperature coefficient
    #[inline]
    pub const fn with_temperature_coefficient(mut self, coeff: f32) -> Self {
        self.temperature_coefficient = coeff;
        self
    }

    /// Set battery age
    #[inline]
    pub const fn with_age_years(mut self, years: f32) -> Self {
        self.age_years = years;
        self
    }

    /// Set aging factor
    #[inline]
    pub const fn with_aging_factor(mut self, factor: f32) -> Self {
        self.aging_factor = factor;
        self
    }

    /// Whether temperature compensation is enabled
    #[inline]
    pub const fn enable_temperature_compensation(self) -> bool {
        (self.flags & 0x01) != 0
    }

    /// Whether aging compensation is enabled
    #[inline]
    pub const fn enable_aging_compensation(self) -> bool {
        (self.flags & 0x02) != 0
    }
}

// Non-const Default implementation
impl Default for EstimatorConfig {
    #[inline]
    fn default() -> Self {
        Self::default()
    }
}

/// SOC estimator
#[derive(Debug, Clone, Copy)]
pub struct SocEstimator {
    curve: &'static Curve,
    config: EstimatorConfig,
}

impl SocEstimator {
    /// Create a new SOC estimator (default configuration)
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

    /// Create estimator with custom curve
    pub const fn with_custom_curve(curve: &'static Curve) -> Self {
        Self {
            curve,
            config: EstimatorConfig::default(),
        }
    }

    /// Create estimator with configuration (const version)
    pub const fn with_config(chemistry: BatteryChemistry, config: EstimatorConfig) -> Self {
        let curve = match chemistry {
            BatteryChemistry::LiPo => &default_curves::LIPO,
            BatteryChemistry::LiFePO4 => &default_curves::LIFEPO4,
            BatteryChemistry::LiIon => &default_curves::LIION,
            BatteryChemistry::Lipo410Full340Cutoff => &default_curves::LIPO410_FULL340_CUTOFF,
        };

        Self { curve, config }
    }

    /// Create estimator with configuration
    pub fn with_config_non_const(chemistry: BatteryChemistry, config: EstimatorConfig) -> Self {
        Self::with_config(chemistry, config)
    }

    /// Estimate SOC (without temperature compensation)
    pub fn estimate_soc(&self, voltage: f32) -> Result<f32, Error> {
        self.curve.voltage_to_soc(voltage)
    }

    /// Estimate SOC (with temperature compensation) - Always applies temperature compensation, ignoring configuration
    pub fn estimate_soc_with_temp(&self, voltage: f32, temperature: f32) -> Result<f32, Error> {
        let base_soc = self.curve.voltage_to_soc(voltage)?;

        // Always apply temperature compensation with default parameters
        let compensated = default_temperature_compensation(base_soc, temperature);

        Ok(compensated.clamp(0.0, 100.0))
    }

    /// Estimate SOC (using configuration settings)
    #[inline]
    pub fn estimate_soc_compensated(&self, voltage: f32, temperature: f32) -> Result<f32, Error> {
        let base_soc = self.curve.voltage_to_soc(voltage)?;
        let mut soc = base_soc;

        // Apply temperature compensation
        if EstimatorConfig::enable_temperature_compensation(self.config) {
            soc = compensate_temperature(
                soc,
                temperature,
                self.config.nominal_temperature,
                self.config.temperature_coefficient,
            );
        }

        // Apply aging compensation
        if EstimatorConfig::enable_aging_compensation(self.config) {
            soc = compensate_aging(soc, self.config.age_years, self.config.aging_factor);
        }

        // Ensure SOC is within valid range
        Ok(soc.clamp(0.0, 100.0))
    }

    /// Get voltage range
    pub const fn voltage_range(&self) -> (f32, f32) {
        self.curve.voltage_range()
    }

    /// Update configuration
    #[inline]
    pub fn update_config(&mut self, config: EstimatorConfig) {
        self.config = config;
    }

    /// Get current configuration
    #[inline]
    pub const fn config(&self) -> &EstimatorConfig {
        &self.config
    }

    /// Enable temperature compensation
    #[inline]
    pub fn enable_temperature_compensation(&mut self, nominal_temp: f32, coefficient: f32) {
        self.config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(nominal_temp)
            .with_temperature_coefficient(coefficient);
    }

    /// Enable aging compensation
    #[inline]
    pub fn enable_aging_compensation(&mut self, age_years: f32, aging_factor: f32) {
        self.config = EstimatorConfig::default()
            .with_aging_compensation()
            .with_age_years(age_years)
            .with_aging_factor(aging_factor);
    }

    /// Disable all compensation
    #[inline]
    pub fn disable_all_compensation(&mut self) {
        self.config = EstimatorConfig::default();
    }
}

// Convenience constructors for simplified usage
impl SocEstimator {
    /// Create estimator with temperature compensation
    #[inline]
    pub fn with_temperature_compensation(
        chemistry: BatteryChemistry,
        nominal_temp: f32,
        coefficient: f32,
    ) -> Self {
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(nominal_temp)
            .with_temperature_coefficient(coefficient);

        Self::with_config(chemistry, config)
    }

    /// Create estimator with aging compensation
    #[inline]
    pub fn with_aging_compensation(
        chemistry: BatteryChemistry,
        age_years: f32,
        aging_factor: f32,
    ) -> Self {
        let config = EstimatorConfig::default()
            .with_aging_compensation()
            .with_age_years(age_years)
            .with_aging_factor(aging_factor);

        Self::with_config(chemistry, config)
    }

    /// 创建带所有补偿的估算器
    #[inline]
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

        Self::with_config(chemistry, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimator_basic() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        // Test boundaries
        assert!(estimator.estimate_soc(3.2).unwrap().abs() < 1.0);
        assert!(estimator.estimate_soc(4.2).unwrap() > 99.0);

        // Test typical values
        let soc = estimator.estimate_soc(3.7).unwrap();
        assert!(
            (45.0..=55.0).contains(&soc),
            "3.7V should be around 50%, got {}",
            soc
        );
    }

    #[test]
    fn test_estimator_with_temp() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        // Test different temperatures
        let base_soc = estimator.estimate_soc(3.7).unwrap();
        let cold_soc = estimator.estimate_soc_with_temp(3.7, 0.0).unwrap();
        let hot_soc = estimator.estimate_soc_with_temp(3.7, 50.0).unwrap();

        // Low temperature should show higher SOC
        assert!(cold_soc > base_soc, "Cold temp should increase SOC");
        // High temperature should show lower SOC
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
