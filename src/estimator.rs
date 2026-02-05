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
            temperature_coefficient: 0.005, // 0.5% per °C (matches default_temperature_compensation)
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

    /// Returns `true` if temperature compensation is enabled
    pub const fn is_temperature_compensation_enabled(self) -> bool {
        (self.flags & 0x01) != 0
    }

    /// Returns `true` if aging compensation is enabled
    pub const fn is_aging_compensation_enabled(self) -> bool {
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
            config: EstimatorConfig::default(), // This is now a const function
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

    /// Estimate SOC (without temperature compensation)
    pub fn estimate_soc(&self, voltage: f32) -> Result<f32, Error> {
        self.curve.voltage_to_soc(voltage)
    }

    /// Estimate SOC with default temperature compensation (ignores configuration)
    ///
    /// This method always applies temperature compensation using default parameters
    /// (nominal temperature: 25°C, coefficient: 0.005), regardless of the estimator's
    /// current configuration. For configuration-based compensation, use
    /// `estimate_soc_compensated()` instead.
    ///
    /// # Arguments
    ///
    /// * `voltage` - Battery voltage in volts
    /// * `temperature` - Current battery temperature in Celsius
    ///
    /// # Returns
    ///
    /// Temperature-compensated SOC percentage using default parameters
    pub fn estimate_soc_with_temp(&self, voltage: f32, temperature: f32) -> Result<f32, Error> {
        let base_soc = self.curve.voltage_to_soc(voltage)?;

        // Always apply temperature compensation with default parameters
        let compensated = default_temperature_compensation(base_soc, temperature);

        Ok(compensated.clamp(0.0, 100.0))
    }

    /// Estimate SOC (using configuration settings)
    pub fn estimate_soc_compensated(&self, voltage: f32, temperature: f32) -> Result<f32, Error> {
        let base_soc = self.curve.voltage_to_soc(voltage)?;
        let mut soc = base_soc;

        // Apply temperature compensation
        if EstimatorConfig::is_temperature_compensation_enabled(self.config) {
            soc = compensate_temperature(
                soc,
                temperature,
                self.config.nominal_temperature,
                self.config.temperature_coefficient,
            );
        }

        // Apply aging compensation
        if EstimatorConfig::is_aging_compensation_enabled(self.config) {
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
    pub fn enable_temperature_compensation(&mut self, nominal_temp: f32, coefficient: f32) {
        self.config = self
            .config
            .with_temperature_compensation()
            .with_nominal_temperature(nominal_temp)
            .with_temperature_coefficient(coefficient);
    }

    /// Enable aging compensation
    pub fn enable_aging_compensation(&mut self, age_years: f32, aging_factor: f32) {
        self.config = self
            .config
            .with_aging_compensation()
            .with_age_years(age_years)
            .with_aging_factor(aging_factor);
    }

    /// Disable all compensation
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

    /// Create estimator with all compensation enabled
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

        // Low temperature should show LOWER SOC (reduced capacity due to higher internal resistance)
        assert!(
            cold_soc < base_soc,
            "Cold temp should decrease SOC due to reduced capacity"
        );
        // High temperature should show slightly higher SOC (better efficiency)
        assert!(
            hot_soc >= base_soc,
            "Hot temp should maintain or slightly increase SOC"
        );
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
    fn test_estimator_estimate_soc_compensated() {
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_aging_compensation()
            .with_age_years(1.0)
            .with_aging_factor(0.02);

        let estimator = SocEstimator::with_config(BatteryChemistry::LiPo, config);

        // Test with both compensations enabled
        let soc = estimator.estimate_soc_compensated(3.7, 25.0).unwrap();
        assert!(soc > 0.0 && soc < 100.0);

        // Cold temperature should reduce SOC
        let cold_soc = estimator.estimate_soc_compensated(3.7, 0.0).unwrap();
        assert!(cold_soc < soc);
    }

    #[test]
    fn test_estimator_update_config() {
        let mut estimator = SocEstimator::new(BatteryChemistry::LiPo);

        let new_config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(30.0);

        estimator.update_config(new_config);
        assert!(estimator.config().is_temperature_compensation_enabled());
        assert_eq!(estimator.config().nominal_temperature, 30.0);
    }

    #[test]
    fn test_estimator_with_all_compensation() {
        let estimator =
            SocEstimator::with_all_compensation(BatteryChemistry::LiPo, 25.0, 0.005, 2.0, 0.02);

        let config = estimator.config();
        assert!(config.is_temperature_compensation_enabled());
        assert!(config.is_aging_compensation_enabled());
        assert_eq!(config.nominal_temperature, 25.0);
        assert_eq!(config.temperature_coefficient, 0.005);
        assert_eq!(config.age_years, 2.0);
        assert_eq!(config.aging_factor, 0.02);
    }

    #[test]
    fn test_estimator_with_config_lipo410() {
        // Test with_config using Lipo410Full340Cutoff to cover line 137
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(25.0);

        let estimator = SocEstimator::with_config(BatteryChemistry::Lipo410Full340Cutoff, config);

        // Verify the curve is correct
        let (min, max) = estimator.voltage_range();
        assert_eq!(min, 3.4);
        assert_eq!(max, 4.1);

        // Test SOC estimation
        let soc = estimator.estimate_soc(3.77).unwrap();
        assert!((soc - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_estimate_soc_compensated_with_temp_only() {
        // Test temperature compensation in estimate_soc_compensated
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(25.0)
            .with_temperature_coefficient(0.005); // 0.5% per °C

        let estimator = SocEstimator::with_config(BatteryChemistry::LiPo, config);

        // At cold temperature (0°C), SOC should appear LOWER (reduced capacity)
        let soc_cold = estimator.estimate_soc_compensated(3.7, 0.0).unwrap();
        let soc_normal = estimator.estimate_soc_compensated(3.7, 25.0).unwrap();

        assert!(
            soc_cold < soc_normal,
            "Cold temperature should decrease SOC due to reduced capacity"
        );
    }

    #[test]
    fn test_estimator_disable_all_compensation() {
        let mut estimator =
            SocEstimator::with_all_compensation(BatteryChemistry::LiPo, 25.0, 0.0005, 2.0, 0.02);

        estimator.disable_all_compensation();

        assert!(!estimator.config().is_temperature_compensation_enabled());
        assert!(!estimator.config().is_aging_compensation_enabled());
    }

    #[test]
    fn test_estimator_enable_methods() {
        // Test enable_temperature_compensation method (lines 212-217)
        let mut estimator = SocEstimator::new(BatteryChemistry::LiPo);
        estimator.enable_temperature_compensation(30.0, 0.006);
        assert!(estimator.config().is_temperature_compensation_enabled());
        assert_eq!(estimator.config().nominal_temperature, 30.0);
        assert_eq!(estimator.config().temperature_coefficient, 0.006);

        // Test enable_aging_compensation method (lines 221-226)
        estimator.enable_aging_compensation(3.0, 0.03);
        assert!(estimator.config().is_aging_compensation_enabled());
        assert_eq!(estimator.config().age_years, 3.0);
        assert_eq!(estimator.config().aging_factor, 0.03);
    }

    #[test]
    fn test_estimator_convenience_constructors() {
        // Test with_temperature_compensation (lines 239-249)
        let estimator1 =
            SocEstimator::with_temperature_compensation(BatteryChemistry::LiPo, 30.0, 0.006);
        assert!(estimator1.config().is_temperature_compensation_enabled());
        assert_eq!(estimator1.config().nominal_temperature, 30.0);
        assert_eq!(estimator1.config().temperature_coefficient, 0.006);

        // Test with_aging_compensation (lines 254-264)
        let estimator2 =
            SocEstimator::with_aging_compensation(BatteryChemistry::LiFePO4, 2.0, 0.025);
        assert!(estimator2.config().is_aging_compensation_enabled());
        assert_eq!(estimator2.config().age_years, 2.0);
        assert_eq!(estimator2.config().aging_factor, 0.025);

        // Test with_config for all battery chemistries including LiIon (line 134)
        let lilon_config = EstimatorConfig::default();
        let lilon_estimator = SocEstimator::with_config(BatteryChemistry::LiIon, lilon_config);
        let (min, max) = lilon_estimator.voltage_range();
        assert_eq!(min, 2.5); // LiIon min voltage is 2.5V
        assert_eq!(max, 4.2);

        // Test Default trait for EstimatorConfig (lines 93-94)
        let default_config: EstimatorConfig = Default::default();
        assert_eq!(default_config.nominal_temperature, 25.0);
        assert_eq!(default_config.temperature_coefficient, 0.005);
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
}
