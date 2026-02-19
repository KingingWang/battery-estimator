//! SOC (State of Charge) Estimator with Temperature Compensation

use crate::curve::default_curves;
use crate::{
    compensate_aging_fixed, compensate_temperature_fixed, default_temperature_compensation_fixed,
    BatteryChemistry, Curve, Error, Fixed,
};

/// SOC estimator configuration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EstimatorConfig {
    /// Nominal temperature (°C) as fixed-point
    pub nominal_temperature: Fixed,
    /// Temperature compensation coefficient (percentage change per °C) as fixed-point
    pub temperature_coefficient: Fixed,
    /// Battery age (years) as fixed-point
    pub age_years: Fixed,
    /// Aging factor (capacity loss percentage per year) as fixed-point
    pub aging_factor: Fixed,
    /// Compensation flags (bit field compression)
    flags: u8,
}

impl EstimatorConfig {
    /// Default configuration
    #[inline]
    pub const fn default() -> Self {
        Self {
            nominal_temperature: Fixed::from_bits(25 << 16), // 25.0
            temperature_coefficient: Fixed::from_bits(328),  // 0.005
            age_years: Fixed::ZERO,
            aging_factor: Fixed::from_bits(1311), // 0.02
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
    pub fn with_nominal_temperature(mut self, temp: Fixed) -> Self {
        self.nominal_temperature = temp;
        self
    }

    /// Set temperature coefficient
    #[inline]
    pub fn with_temperature_coefficient(mut self, coeff: Fixed) -> Self {
        self.temperature_coefficient = coeff;
        self
    }

    /// Set battery age
    #[inline]
    pub fn with_age_years(mut self, years: Fixed) -> Self {
        self.age_years = years;
        self
    }

    /// Set aging factor
    #[inline]
    pub fn with_aging_factor(mut self, factor: Fixed) -> Self {
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
            config: EstimatorConfig::default(),
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

    /// Estimate SOC using fixed-point arithmetic (without temperature compensation)
    ///
    /// # Arguments
    ///
    /// * `voltage` - Battery voltage as fixed-point value
    ///
    /// # Returns
    ///
    /// * `Ok(soc)` - SOC percentage as fixed-point value
    /// * `Err(Error)` - Error if estimation fails
    pub fn estimate_soc_fixed(&self, voltage: Fixed) -> Result<Fixed, Error> {
        self.curve.voltage_to_soc_fixed(voltage)
    }

    /// Estimate SOC (without temperature compensation)
    pub fn estimate_soc(&self, voltage: f32) -> Result<f32, Error> {
        self.curve.voltage_to_soc(voltage)
    }

    /// Estimate SOC with default temperature compensation using fixed-point arithmetic
    ///
    /// This method always applies temperature compensation using default parameters
    /// (nominal temperature: 25°C, coefficient: 0.005), regardless of the estimator's
    /// current configuration.
    ///
    /// # Arguments
    ///
    /// * `voltage` - Battery voltage as fixed-point value
    /// * `temperature` - Current battery temperature in Celsius as fixed-point
    ///
    /// # Returns
    ///
    /// Temperature-compensated SOC percentage using default parameters
    pub fn estimate_soc_with_temp_fixed(
        &self,
        voltage: Fixed,
        temperature: Fixed,
    ) -> Result<Fixed, Error> {
        let base_soc = self.curve.voltage_to_soc_fixed(voltage)?;
        let compensated = default_temperature_compensation_fixed(base_soc, temperature);
        Ok(compensated.clamp(Fixed::ZERO, Fixed::from_num(100)))
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
        let compensated = default_temperature_compensation_fixed(
            Fixed::from_num(base_soc),
            Fixed::from_num(temperature),
        );

        Ok(compensated
            .clamp(Fixed::ZERO, Fixed::from_num(100))
            .to_num::<f32>())
    }

    /// Estimate SOC using configuration settings with fixed-point arithmetic
    ///
    /// # Arguments
    ///
    /// * `voltage` - Battery voltage as fixed-point value
    /// * `temperature` - Current battery temperature in Celsius as fixed-point
    ///
    /// # Returns
    ///
    /// Compensated SOC percentage as fixed-point value
    pub fn estimate_soc_compensated_fixed(
        &self,
        voltage: Fixed,
        temperature: Fixed,
    ) -> Result<Fixed, Error> {
        let base_soc = self.curve.voltage_to_soc_fixed(voltage)?;
        let mut soc = base_soc;

        if self.config.is_temperature_compensation_enabled() {
            soc = compensate_temperature_fixed(
                soc,
                temperature,
                self.config.nominal_temperature,
                self.config.temperature_coefficient,
            );
        }

        if self.config.is_aging_compensation_enabled() {
            soc = compensate_aging_fixed(soc, self.config.age_years, self.config.aging_factor);
        }

        Ok(soc.clamp(Fixed::ZERO, Fixed::from_num(100)))
    }

    /// Estimate SOC (using configuration settings)
    pub fn estimate_soc_compensated(&self, voltage: f32, temperature: f32) -> Result<f32, Error> {
        let result = self.estimate_soc_compensated_fixed(
            Fixed::from_num(voltage),
            Fixed::from_num(temperature),
        )?;
        Ok(result.to_num::<f32>())
    }

    /// Get voltage range
    pub const fn voltage_range(&self) -> (f32, f32) {
        self.curve.voltage_range()
    }

    /// Get voltage range as fixed-point values
    pub fn voltage_range_fixed(&self) -> (Fixed, Fixed) {
        self.curve.voltage_range_fixed()
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
    pub fn enable_temperature_compensation(&mut self, nominal_temp: Fixed, coefficient: Fixed) {
        self.config = self
            .config
            .with_temperature_compensation()
            .with_nominal_temperature(nominal_temp)
            .with_temperature_coefficient(coefficient);
    }

    /// Enable aging compensation
    pub fn enable_aging_compensation(&mut self, age_years: Fixed, aging_factor: Fixed) {
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
        nominal_temp: Fixed,
        coefficient: Fixed,
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
        age_years: Fixed,
        aging_factor: Fixed,
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
        nominal_temp: Fixed,
        temp_coeff: Fixed,
        age_years: Fixed,
        aging_factor: Fixed,
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
    fn test_estimator_fixed() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        // Test boundaries
        let soc_min = estimator.estimate_soc_fixed(Fixed::from_num(3.2)).unwrap();
        assert!(soc_min < Fixed::from_num(1.0));

        let soc_max = estimator.estimate_soc_fixed(Fixed::from_num(4.2)).unwrap();
        assert!(soc_max > Fixed::from_num(99.0));

        // Test typical values
        let soc = estimator.estimate_soc_fixed(Fixed::from_num(3.7)).unwrap();
        assert!(soc > Fixed::from_num(45.0) && soc < Fixed::from_num(55.0));
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
    fn test_estimator_with_temp_fixed() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        let base_soc = estimator.estimate_soc_fixed(Fixed::from_num(3.7)).unwrap();
        let cold_soc = estimator
            .estimate_soc_with_temp_fixed(Fixed::from_num(3.7), Fixed::ZERO)
            .unwrap();
        let hot_soc = estimator
            .estimate_soc_with_temp_fixed(Fixed::from_num(3.7), Fixed::from_num(50.0))
            .unwrap();

        // Low temperature should show LOWER SOC
        assert!(cold_soc < base_soc);

        // High temperature should show slightly higher SOC
        assert!(hot_soc >= base_soc);
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
    fn test_estimator_voltage_range_fixed() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        let (min, max) = estimator.voltage_range_fixed();
        assert_eq!(min, Fixed::from_num(3.2));
        assert_eq!(max, Fixed::from_num(4.2));
    }

    #[test]
    fn test_estimator_estimate_soc_compensated() {
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_aging_compensation()
            .with_age_years(Fixed::from_num(1.0))
            .with_aging_factor(Fixed::from_num(0.02));

        let estimator = SocEstimator::with_config(BatteryChemistry::LiPo, config);

        // Test with both compensations enabled
        let soc = estimator.estimate_soc_compensated(3.7, 25.0).unwrap();
        assert!(soc > 0.0 && soc < 100.0);

        // Cold temperature should reduce SOC
        let cold_soc = estimator.estimate_soc_compensated(3.7, 0.0).unwrap();
        assert!(cold_soc < soc);
    }

    #[test]
    fn test_estimator_estimate_soc_compensated_fixed() {
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_aging_compensation()
            .with_age_years(Fixed::from_num(1.0))
            .with_aging_factor(Fixed::from_num(0.02));

        let estimator = SocEstimator::with_config(BatteryChemistry::LiPo, config);

        // Test with both compensations enabled
        let soc = estimator
            .estimate_soc_compensated_fixed(Fixed::from_num(3.7), Fixed::from_num(25.0))
            .unwrap();
        assert!(soc > Fixed::ZERO && soc < Fixed::from_num(100.0));

        // Cold temperature should reduce SOC
        let cold_soc = estimator
            .estimate_soc_compensated_fixed(Fixed::from_num(3.7), Fixed::ZERO)
            .unwrap();
        assert!(cold_soc < soc);
    }

    #[test]
    fn test_estimator_update_config() {
        let mut estimator = SocEstimator::new(BatteryChemistry::LiPo);

        let new_config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(Fixed::from_num(30.0));

        estimator.update_config(new_config);

        assert!(estimator.config().is_temperature_compensation_enabled());
        assert_eq!(
            estimator.config().nominal_temperature,
            Fixed::from_num(30.0)
        );
    }

    #[test]
    fn test_estimator_with_all_compensation() {
        let estimator = SocEstimator::with_all_compensation(
            BatteryChemistry::LiPo,
            Fixed::from_num(25.0),
            Fixed::from_num(0.005),
            Fixed::from_num(2.0),
            Fixed::from_num(0.02),
        );

        let config = estimator.config();
        assert!(config.is_temperature_compensation_enabled());
        assert!(config.is_aging_compensation_enabled());
        assert_eq!(config.nominal_temperature, Fixed::from_num(25.0));
        assert_eq!(config.temperature_coefficient, Fixed::from_num(0.005));
        assert_eq!(config.age_years, Fixed::from_num(2.0));
        assert_eq!(config.aging_factor, Fixed::from_num(0.02));
    }

    #[test]
    fn test_estimator_with_config_lipo410() {
        // Test with_config using Lipo410Full340Cutoff to cover line 137
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(Fixed::from_num(25.0));

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
            .with_nominal_temperature(Fixed::from_num(25.0))
            .with_temperature_coefficient(Fixed::from_num(0.005)); // 0.5% per °C

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
        let mut estimator = SocEstimator::with_all_compensation(
            BatteryChemistry::LiPo,
            Fixed::from_num(25.0),
            Fixed::from_num(0.0005),
            Fixed::from_num(2.0),
            Fixed::from_num(0.02),
        );

        estimator.disable_all_compensation();

        assert!(!estimator.config().is_temperature_compensation_enabled());
        assert!(!estimator.config().is_aging_compensation_enabled());
    }

    #[test]
    fn test_estimator_enable_methods() {
        // Test enable_temperature_compensation method
        let mut estimator = SocEstimator::new(BatteryChemistry::LiPo);

        estimator.enable_temperature_compensation(Fixed::from_num(30.0), Fixed::from_num(0.006));

        assert!(estimator.config().is_temperature_compensation_enabled());
        assert_eq!(
            estimator.config().nominal_temperature,
            Fixed::from_num(30.0)
        );
        assert_eq!(
            estimator.config().temperature_coefficient,
            Fixed::from_num(0.006)
        );

        // Test enable_aging_compensation method
        estimator.enable_aging_compensation(Fixed::from_num(3.0), Fixed::from_num(0.03));

        assert!(estimator.config().is_aging_compensation_enabled());
        assert_eq!(estimator.config().age_years, Fixed::from_num(3.0));
        assert_eq!(estimator.config().aging_factor, Fixed::from_num(0.03));
    }

    #[test]
    fn test_estimator_convenience_constructors() {
        // Test with_temperature_compensation
        let estimator1 = SocEstimator::with_temperature_compensation(
            BatteryChemistry::LiPo,
            Fixed::from_num(30.0),
            Fixed::from_num(0.006),
        );

        assert!(estimator1.config().is_temperature_compensation_enabled());
        assert_eq!(
            estimator1.config().nominal_temperature,
            Fixed::from_num(30.0)
        );
        assert_eq!(
            estimator1.config().temperature_coefficient,
            Fixed::from_num(0.006)
        );

        // Test with_aging_compensation
        let estimator2 = SocEstimator::with_aging_compensation(
            BatteryChemistry::LiFePO4,
            Fixed::from_num(2.0),
            Fixed::from_num(0.025),
        );

        assert!(estimator2.config().is_aging_compensation_enabled());
        assert_eq!(estimator2.config().age_years, Fixed::from_num(2.0));
        assert_eq!(estimator2.config().aging_factor, Fixed::from_num(0.025));

        // Test with_config for all battery chemistries including LiIon
        let lilon_config = EstimatorConfig::default();
        let lilon_estimator = SocEstimator::with_config(BatteryChemistry::LiIon, lilon_config);

        let (min, max) = lilon_estimator.voltage_range();
        assert_eq!(min, 2.5); // LiIon min voltage is 2.5V
        assert_eq!(max, 4.2);

        // Test Default trait for EstimatorConfig
        let default_config: EstimatorConfig = Default::default();
        assert_eq!(default_config.nominal_temperature, Fixed::from_num(25.0));
        assert_eq!(
            default_config.temperature_coefficient,
            Fixed::from_num(0.005)
        );
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
    fn test_estimator_config_default_values() {
        let config = EstimatorConfig::default();

        // Check default values
        assert_eq!(config.nominal_temperature, Fixed::from_num(25.0));
        assert_eq!(config.temperature_coefficient, Fixed::from_num(0.005));
        assert_eq!(config.age_years, Fixed::ZERO);
        assert_eq!(config.aging_factor, Fixed::from_num(0.02));
        assert!(!config.is_temperature_compensation_enabled());
        assert!(!config.is_aging_compensation_enabled());
    }

    #[test]
    fn test_estimator_config_flags() {
        let config = EstimatorConfig::default().with_temperature_compensation();

        assert!(config.is_temperature_compensation_enabled());
        assert!(!config.is_aging_compensation_enabled());

        let config = config.with_aging_compensation();

        assert!(config.is_temperature_compensation_enabled());
        assert!(config.is_aging_compensation_enabled());
    }

    #[test]
    fn test_estimator_fixed_point_precision() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        // Test that fixed-point calculations maintain precision
        let voltage = Fixed::from_num(3.75);
        let soc = estimator.estimate_soc_fixed(voltage).unwrap();

        // SOC should be approximately 60% at 3.75V for LiPo
        assert!(soc > Fixed::from_num(55.0) && soc < Fixed::from_num(65.0));
    }
}
