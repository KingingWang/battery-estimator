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
    #[test]
    fn test_estimator_all_battery_types() {
        // Test all battery chemistries
        let lipo = SocEstimator::new(BatteryChemistry::LiPo);
        let lifepo4 = SocEstimator::new(BatteryChemistry::LiFePO4);
        let _lilon = SocEstimator::new(BatteryChemistry::LiIon);
        let conservative = SocEstimator::new(BatteryChemistry::Lipo410Full340Cutoff);

        // All should produce valid SOC values
        assert!(lipo.estimate_soc(3.7).is_ok());
        assert!(lifepo4.estimate_soc(3.2).is_ok());
        assert!(_lilon.estimate_soc(3.7).is_ok());
        assert!(conservative.estimate_soc(3.77).is_ok());
    }
    #[test]
    fn test_estimator_voltage_range() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);
        let (min, max) = estimator.voltage_range();

        assert_eq!(min, 3.2);
        assert_eq!(max, 4.2);
    }

    #[test]
    fn test_estimator_config_default() {
        let config = EstimatorConfig::default();

        assert_eq!(config.nominal_temperature, 25.0);
        assert_eq!(config.temperature_coefficient, 0.0005);
        assert_eq!(config.age_years, 0.0);
        assert_eq!(config.aging_factor, 0.02);
        assert!(!config.enable_temperature_compensation());
        assert!(!config.enable_aging_compensation());
    }

    #[test]
    fn test_estimator_config_builder() {
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(20.0)
            .with_temperature_coefficient(0.001)
            .with_age_years(2.0)
            .with_aging_factor(0.03);

        assert!(config.enable_temperature_compensation());
        assert_eq!(config.nominal_temperature, 20.0);
        assert_eq!(config.temperature_coefficient, 0.001);
        assert_eq!(config.age_years, 2.0);
        assert_eq!(config.aging_factor, 0.03);
    }

    #[test]
    fn test_estimator_with_temperature_compensation() {
        let estimator =
            SocEstimator::with_temperature_compensation(BatteryChemistry::LiPo, 25.0, 0.0005);

        assert!(estimator.config().enable_temperature_compensation());
        assert_eq!(estimator.config().nominal_temperature, 25.0);
        assert_eq!(estimator.config().temperature_coefficient, 0.0005);
    }

    #[test]
    fn test_estimator_with_aging_compensation() {
        let estimator = SocEstimator::with_aging_compensation(BatteryChemistry::LiPo, 3.0, 0.02);

        assert!(estimator.config().enable_aging_compensation());
        assert_eq!(estimator.config().age_years, 3.0);
        assert_eq!(estimator.config().aging_factor, 0.02);
    }

    #[test]
    fn test_estimator_with_all_compensation() {
        let estimator =
            SocEstimator::with_all_compensation(BatteryChemistry::LiPo, 25.0, 0.0005, 2.0, 0.02);

        assert!(estimator.config().enable_temperature_compensation());
        assert!(estimator.config().enable_aging_compensation());
        assert_eq!(estimator.config().nominal_temperature, 25.0);
        assert_eq!(estimator.config().age_years, 2.0);
    }

    #[test]
    fn test_estimator_update_config() {
        let mut estimator = SocEstimator::new(BatteryChemistry::LiPo);

        let new_config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(20.0);

        estimator.update_config(new_config);

        assert!(estimator.config().enable_temperature_compensation());
        assert_eq!(estimator.config().nominal_temperature, 20.0);
    }

    #[test]
    fn test_estimator_enable_temperature_compensation() {
        let mut estimator = SocEstimator::new(BatteryChemistry::LiPo);

        estimator.enable_temperature_compensation(20.0, 0.001);

        assert!(estimator.config().enable_temperature_compensation());
        assert_eq!(estimator.config().nominal_temperature, 20.0);
        assert_eq!(estimator.config().temperature_coefficient, 0.001);
    }

    #[test]
    fn test_estimator_enable_aging_compensation() {
        let mut estimator = SocEstimator::new(BatteryChemistry::LiPo);

        estimator.enable_aging_compensation(5.0, 0.03);

        assert!(estimator.config().enable_aging_compensation());
        assert_eq!(estimator.config().age_years, 5.0);
        assert_eq!(estimator.config().aging_factor, 0.03);
    }

    #[test]
    fn test_estimator_config_default_trait() {
        // Test Default trait implementation for EstimatorConfig (line 81)
        let config: EstimatorConfig = Default::default();

        assert_eq!(config.nominal_temperature, 25.0);
        assert_eq!(config.temperature_coefficient, 0.0005);
        assert_eq!(config.age_years, 0.0);
        assert_eq!(config.aging_factor, 0.02);
    }

    #[test]
    fn test_estimator_config_enable_methods() {
        // Test enable_temperature_compensation and enable_aging_compensation methods (lines 87, 95-96)
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_aging_compensation();

        // Test enable_temperature_compensation method (line 87)
        assert!(config.enable_temperature_compensation());

        // Test enable_aging_compensation method (lines 95-96)
        assert!(config.enable_aging_compensation());
    }

    #[test]
    fn test_estimate_soc_compensated_with_temp_only() {
        // Test temperature compensation in estimate_soc_compensated (lines 135-137)
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(25.0)
            .with_temperature_coefficient(0.0005);

        let estimator = SocEstimator::with_config(BatteryChemistry::LiPo, config);

        // At cold temperature (0°C), SOC should appear higher
        let soc_cold = estimator.estimate_soc_compensated(3.7, 0.0).unwrap();
        let soc_normal = estimator.estimate_soc_compensated(3.7, 25.0).unwrap();

        assert!(
            soc_cold > soc_normal,
            "Cold temperature should increase apparent SOC"
        );
    }

    #[test]
    fn test_estimate_soc_compensated_with_aging_only() {
        // Test aging compensation in estimate_soc_compensated (line 173)
        let config = EstimatorConfig::default()
            .with_aging_compensation()
            .with_age_years(2.0)
            .with_aging_factor(0.02);

        let estimator = SocEstimator::with_config(BatteryChemistry::LiPo, config);

        // Aged battery should show lower SOC
        let soc_aged = estimator.estimate_soc_compensated(3.7, 25.0).unwrap();

        // Create estimator without aging for comparison
        let config_new = EstimatorConfig::default();
        let estimator_new = SocEstimator::with_config(BatteryChemistry::LiPo, config_new);
        let soc_new = estimator_new.estimate_soc_compensated(3.7, 25.0).unwrap();

        assert!(soc_aged < soc_new, "Aged battery should show lower SOC");
    }

    #[test]
    fn test_estimator_disable_all_compensation() {
        let mut estimator =
            SocEstimator::with_all_compensation(BatteryChemistry::LiPo, 25.0, 0.0005, 2.0, 0.02);

        estimator.disable_all_compensation();

        assert!(!estimator.config().enable_temperature_compensation());
        assert!(!estimator.config().enable_aging_compensation());
    }

    #[test]
    fn test_estimate_soc_compensated_with_temp() {
        let estimator =
            SocEstimator::with_temperature_compensation(BatteryChemistry::LiPo, 25.0, 0.0005);

        let base_soc = estimator.estimate_soc(3.7).unwrap();
        let compensated_soc = estimator.estimate_soc_compensated(3.7, 20.0).unwrap();

        // At lower temperature, SOC should appear higher
        assert!(compensated_soc > base_soc);
    }

    #[test]
    fn test_estimate_soc_compensated_with_aging() {
        let estimator = SocEstimator::with_aging_compensation(BatteryChemistry::LiPo, 3.0, 0.02);

        let base_soc = estimator.estimate_soc(3.7).unwrap();
        let compensated_soc = estimator.estimate_soc_compensated(3.7, 25.0).unwrap();

        // With aging, SOC should appear lower
        assert!(compensated_soc < base_soc);
    }

    #[test]
    fn test_estimate_soc_compensated_with_all() {
        let estimator =
            SocEstimator::with_all_compensation(BatteryChemistry::LiPo, 25.0, 0.0005, 3.0, 0.02);

        // Should apply both temperature and aging compensation
        let result = estimator.estimate_soc_compensated(3.7, 20.0);
        assert!(result.is_ok());

        let soc = result.unwrap();
        assert!((0.0..=100.0).contains(&soc));
    }

    #[test]
    fn test_estimate_soc_no_compensation() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        let base_soc = estimator.estimate_soc(3.7).unwrap();
        let compensated_soc = estimator.estimate_soc_compensated(3.7, 20.0).unwrap();

        // Without compensation enabled, should be the same
        assert_eq!(base_soc, compensated_soc);
    }

    #[test]
    fn test_estimate_soc_with_temp_clamping() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        // Test that temperature compensation is clamped to valid range
        let result = estimator.estimate_soc_with_temp(3.7, -100.0);
        assert!(result.is_ok());

        let soc = result.unwrap();
        assert!((0.0..=100.0).contains(&soc));
    }

    #[test]
    fn test_estimator_copy() {
        let estimator1 = SocEstimator::new(BatteryChemistry::LiPo);
        let estimator2 = estimator1;

        // Both should work independently
        assert!(estimator1.estimate_soc(3.7).is_ok());
        assert!(estimator2.estimate_soc(3.7).is_ok());
    }

    #[test]
    fn test_estimator_extreme_temperatures() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        // Test extreme cold
        let cold_result = estimator.estimate_soc_with_temp(3.7, -40.0);
        assert!(cold_result.is_ok());

        // Test extreme heat
        let hot_result = estimator.estimate_soc_with_temp(3.7, 80.0);
        assert!(hot_result.is_ok());

        // Results should be clamped to valid range
        assert!(cold_result.unwrap() >= 0.0 && cold_result.unwrap() <= 100.0);
        assert!(hot_result.unwrap() >= 0.0 && hot_result.unwrap() <= 100.0);
    }

    #[test]
    fn test_estimator_boundary_voltages() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        // Test at minimum voltage
        let min_result = estimator.estimate_soc(3.2);
        assert!(min_result.is_ok());
        assert!(min_result.unwrap() < 5.0);

        // Test at maximum voltage
        let max_result = estimator.estimate_soc(4.2);
        assert!(max_result.is_ok());
        assert!(max_result.unwrap() > 95.0);
    }

    #[test]
    fn test_estimator_with_config() {
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(20.0);

        let estimator = SocEstimator::with_config(BatteryChemistry::LiPo, config);

        assert!(estimator.config().enable_temperature_compensation());
        assert_eq!(estimator.config().nominal_temperature, 20.0);
    }

    #[test]
    fn test_estimator_with_config_non_const() {
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(20.0);

        let estimator = SocEstimator::with_config_non_const(BatteryChemistry::LiPo, config);

        assert!(estimator.config().enable_temperature_compensation());
        assert_eq!(estimator.config().nominal_temperature, 20.0);
    }

    #[test]
    fn test_estimator_compensated_soc_clamping() {
        let estimator =
            SocEstimator::with_temperature_compensation(BatteryChemistry::LiPo, 25.0, 0.0005);

        // Test that compensated SOC is always clamped to valid range
        let low_result = estimator.estimate_soc_compensated(3.2, -100.0);
        let high_result = estimator.estimate_soc_compensated(4.2, 100.0);

        assert!(low_result.is_ok());
        assert!(high_result.is_ok());

        assert!(low_result.unwrap() >= 0.0);
        assert!(high_result.unwrap() <= 100.0);
    }

    #[test]
    fn test_estimator_different_chemistries_ranges() {
        let lipo = SocEstimator::new(BatteryChemistry::LiPo);
        let lifepo4 = SocEstimator::new(BatteryChemistry::LiFePO4);
        let _lilon = SocEstimator::new(BatteryChemistry::LiIon);
        let conservative = SocEstimator::new(BatteryChemistry::Lipo410Full340Cutoff);

        // Check voltage ranges are different
        assert_ne!(lipo.voltage_range(), lifepo4.voltage_range());
        assert_ne!(lipo.voltage_range(), conservative.voltage_range());
    }

    #[test]
    fn test_estimator_config_flags() {
        // Test that flags are set correctly
        let config1 = EstimatorConfig::default().with_temperature_compensation();
        assert!(config1.enable_temperature_compensation());
        assert!(!config1.enable_aging_compensation());

        let config2 = EstimatorConfig::default().with_aging_compensation();
        assert!(!config2.enable_temperature_compensation());
        assert!(config2.enable_aging_compensation());

        let config3 = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_aging_compensation();
        assert!(config3.enable_temperature_compensation());
        assert!(config3.enable_aging_compensation());
    }

    #[test]
    fn test_estimator_decimal_voltages() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        // Test with decimal voltages
        assert!(estimator.estimate_soc(3.71).is_ok());
        assert!(estimator.estimate_soc(3.99).is_ok());
        assert!(estimator.estimate_soc(3.25).is_ok());
    }

    #[test]
    fn test_estimator_negative_voltage() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        // Negative voltage should return minimum SOC
        let result = estimator.estimate_soc(-1.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_estimator_very_high_voltage() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        // Very high voltage should return maximum SOC
        let result = estimator.estimate_soc(100.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100.0);
    }
}
