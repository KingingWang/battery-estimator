//! Temperature and aging compensation for battery SOC estimation
//!
//! This module provides functions to adjust SOC estimates based on
//! environmental conditions and battery age.

/// Applies temperature compensation to SOC value
///
/// Battery performance varies with temperature. This function adjusts
/// the estimated SOC to account for temperature effects on battery capacity.
///
/// # Arguments
///
/// * `soc` - Base SOC percentage (0.0 to 100.0)
/// * `temperature` - Current battery temperature in Celsius
/// * `nominal_temp` - Nominal/reference temperature in Celsius (typically 25°C)
/// * `coefficient` - Temperature coefficient (change per °C, e.g., 0.0005 = 0.05%/°C)
///
/// # Returns
///
/// Temperature-compensated SOC percentage
///
/// # Behavior
///
/// - At nominal temperature: No adjustment
/// - Below nominal: SOC appears higher (battery less efficient)
/// - Above nominal: SOC appears lower (battery degrades faster)
/// - Compensation is bounded to ±5% maximum
///
/// # Examples
///
/// ```
/// use battery_estimator::compensate_temperature;
///
/// // At nominal temperature (25°C)
/// let soc = compensate_temperature(50.0, 25.0, 25.0, 0.0005);
/// assert_eq!(soc, 50.0);
///
/// // At cold temperature (0°C)
/// let cold_soc = compensate_temperature(50.0, 0.0, 25.0, 0.0005);
/// assert!(cold_soc > 50.0); // SOC appears higher in cold
///
/// // At hot temperature (50°C)
/// let hot_soc = compensate_temperature(50.0, 50.0, 25.0, 0.0005);
/// assert!(hot_soc < 50.0); // SOC appears lower in heat
/// ```
#[inline]
pub fn compensate_temperature(
    soc: f32,
    temperature: f32,
    nominal_temp: f32,
    coefficient: f32,
) -> f32 {
    let delta_temp = temperature - nominal_temp;
    let compensation = delta_temp * coefficient;

    // Limit compensation to ±5% maximum
    let bounded_compensation = clamp(compensation, -0.05, 0.05);

    soc * (1.0 - bounded_compensation)
}

/// Applies aging compensation to SOC value
///
/// Battery capacity degrades over time due to chemical aging.
/// This function adjusts the estimated SOC to account for capacity loss.
///
/// # Arguments
///
/// * `soc` - Base SOC percentage (0.0 to 100.0)
/// * `age_years` - Battery age in years
/// * `aging_factor` - Aging factor (capacity loss per year, e.g., 0.02 = 2%/year)
///
/// # Returns
///
/// Age-compensated SOC percentage
///
/// # Behavior
///
/// - New battery (0 years): No adjustment
/// - Aged battery: SOC appears lower due to reduced capacity
/// - Maximum compensation is 50% (to prevent unrealistic values)
///
/// # Examples
///
/// ```
/// use battery_estimator::compensate_aging;
///
/// // New battery
/// let soc = compensate_aging(50.0, 0.0, 0.02);
/// assert_eq!(soc, 50.0);
///
/// // 2-year-old battery
/// let aged_soc = compensate_aging(50.0, 2.0, 0.02);
/// assert!(aged_soc < 50.0); // Reduced by ~4%
/// ```
#[inline]
pub fn compensate_aging(soc: f32, age_years: f32, aging_factor: f32) -> f32 {
    let age_compensation = age_years * aging_factor;
    soc * (1.0 - clamp(age_compensation, 0.0, 0.5)) // Max 50% compensation
}

/// Applies default temperature compensation
///
/// This is a convenience function that uses standard default values:
/// - Nominal temperature: 25°C
/// - Temperature coefficient: 0.0005 (0.05% per °C)
///
/// # Arguments
///
/// * `soc` - Base SOC percentage (0.0 to 100.0)
/// * `temperature` - Current battery temperature in Celsius
///
/// # Returns
///
/// Temperature-compensated SOC percentage using default parameters
///
/// # Examples
///
/// ```
/// use battery_estimator::default_temperature_compensation;
///
/// // At room temperature
/// let soc = default_temperature_compensation(50.0, 25.0);
/// assert_eq!(soc, 50.0);
///
/// // At cold temperature
/// let cold_soc = default_temperature_compensation(50.0, 0.0);
/// assert!(cold_soc > 50.0);
/// ```
#[inline]
pub fn default_temperature_compensation(soc: f32, temperature: f32) -> f32 {
    const NOMINAL_TEMP: f32 = 25.0;
    const COEFFICIENT: f32 = 0.0005; // 0.05% per °C

    compensate_temperature(soc, temperature, NOMINAL_TEMP, COEFFICIENT)
}

/// Clamps a value between minimum and maximum bounds
///
/// # Arguments
///
/// * `value` - Value to clamp
/// * `min` - Minimum allowed value
/// * `max` - Maximum allowed value
///
/// # Returns
///
/// Clamped value within [min, max] range
#[inline(always)]
fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_compensation() {
        // Room temperature should have no change
        assert_eq!(default_temperature_compensation(50.0, 25.0), 50.0);

        // Low temperature should increase SOC
        let cold_compensated = default_temperature_compensation(50.0, 0.0);
        assert!(cold_compensated > 50.0, "Cold should increase SOC");

        // High temperature should decrease SOC
        let hot_compensated = default_temperature_compensation(50.0, 50.0);
        assert!(hot_compensated < 50.0, "Hot should decrease SOC");
    }

    #[test]
    fn test_temperature_compensation_bounds() {
        // Test boundary limits (±5%)
        let extreme_cold = default_temperature_compensation(50.0, -100.0);
        let extreme_hot = default_temperature_compensation(50.0, 150.0);

        // Should be limited to ±5%
        assert!(extreme_cold <= 50.0 * 1.05);
        assert!(extreme_hot >= 50.0 * 0.95);
    }

    #[test]
    fn test_aging_compensation() {
        // New battery should have no change
        assert_eq!(compensate_aging(50.0, 0.0, 0.02), 50.0);

        // Aged battery should decrease SOC
        let aged = compensate_aging(50.0, 5.0, 0.02);
        assert!(aged < 50.0, "Aging should decrease SOC");

        // Test maximum 50% compensation
        let very_aged = compensate_aging(50.0, 30.0, 0.02);
        assert!(
            very_aged >= 25.0,
            "Should be limited to 50% max compensation"
        );
    }

    #[test]
    fn test_temperature_compensation_negative_temp() {
        // Test very cold temperatures
        let result = compensate_temperature(50.0, -20.0, 25.0, 0.0005);
        assert!(
            result > 50.0,
            "Cold temperature should increase apparent SOC"
        );
    }

    #[test]
    fn test_temperature_compensation_positive_temp() {
        // Test very hot temperatures
        let result = compensate_temperature(50.0, 60.0, 25.0, 0.0005);
        assert!(
            result < 50.0,
            "Hot temperature should decrease apparent SOC"
        );
    }

    #[test]
    fn test_temperature_compensation_different_coefficients() {
        let base_soc = 50.0;
        let temp = 0.0;
        let nominal = 25.0;

        // Test with different temperature coefficients
        let result1 = compensate_temperature(base_soc, temp, nominal, 0.0005);
        let result2 = compensate_temperature(base_soc, temp, nominal, 0.001);
        let result3 = compensate_temperature(base_soc, temp, nominal, 0.0001);

        // Higher coefficient should result in more compensation
        assert!(
            result2 > result1,
            "Higher coefficient should increase compensation"
        );
        assert!(
            result3 < result1,
            "Lower coefficient should decrease compensation"
        );
    }

    #[test]
    fn test_temperature_compensation_different_nominal_temps() {
        let base_soc = 50.0;
        let temp = 0.0;
        let coefficient = 0.0005;

        // Test with different nominal temperatures
        let result1 = compensate_temperature(base_soc, temp, 25.0, coefficient);
        let result2 = compensate_temperature(base_soc, temp, 0.0, coefficient);

        // When nominal temp equals actual temp, no compensation
        let result3 = compensate_temperature(base_soc, 25.0, 25.0, coefficient);
        assert_eq!(result3, base_soc, "No compensation when temps are equal");

        // Different nominal temps should result in different compensation
        assert!(
            result1 != result2,
            "Different nominal temps should give different results"
        );
    }

    #[test]
    fn test_aging_compensation_different_factors() {
        let base_soc = 50.0;
        let age = 5.0;

        // Test with different aging factors
        let result1 = compensate_aging(base_soc, age, 0.02);
        let result2 = compensate_aging(base_soc, age, 0.05);
        let result3 = compensate_aging(base_soc, age, 0.01);

        // Higher aging factor should result in more reduction
        assert!(
            result2 < result1,
            "Higher aging factor should reduce SOC more"
        );
        assert!(
            result3 > result1,
            "Lower aging factor should reduce SOC less"
        );
    }

    #[test]
    fn test_aging_compensation_different_ages() {
        let base_soc = 50.0;
        let factor = 0.02;

        // Test with different battery ages
        let result1 = compensate_aging(base_soc, 1.0, factor);
        let result2 = compensate_aging(base_soc, 5.0, factor);
        let result3 = compensate_aging(base_soc, 10.0, factor);

        // Older battery should have lower SOC
        assert!(result2 < result1, "Older battery should have lower SOC");
        assert!(
            result3 < result2,
            "Even older battery should have even lower SOC"
        );
    }

    #[test]
    fn test_aging_compensation_zero_age() {
        // Test that zero age results in no compensation
        let result = compensate_aging(50.0, 0.0, 0.02);
        assert_eq!(result, 50.0, "Zero age should result in no compensation");
    }

    #[test]
    fn test_aging_compensation_max_limit() {
        // Test that aging compensation is limited to 50%
        let base_soc = 50.0;

        // Very old battery with high aging factor
        let result = compensate_aging(base_soc, 100.0, 1.0);

        // Should be limited to 50% reduction (25.0)
        assert!(result >= 25.0, "Should be limited to 50% max compensation");
    }

    #[test]
    fn test_temperature_compensation_max_limit() {
        // Test that temperature compensation is limited to ±5%
        let base_soc = 50.0;

        // Extreme temperature difference
        let cold_result = compensate_temperature(base_soc, -200.0, 25.0, 0.0005);
        let hot_result = compensate_temperature(base_soc, 200.0, 25.0, 0.0005);

        // Cold: should be at most 5% increase (52.5)
        assert!(
            cold_result <= 52.5,
            "Cold compensation should be limited to +5%"
        );

        // Hot: should be at most 5% decrease (47.5)
        assert!(
            hot_result >= 47.5,
            "Hot compensation should be limited to -5%"
        );
    }

    #[test]
    fn test_compensation_edge_cases() {
        // Test compensation at boundary SOC values
        let zero_soc = 0.0;
        let full_soc = 100.0;

        // Temperature compensation at 0% SOC
        let temp_comp_zero = default_temperature_compensation(zero_soc, 0.0);
        assert_eq!(temp_comp_zero, 0.0, "0% SOC should remain 0%");

        // Temperature compensation at 100% SOC
        let temp_comp_full = default_temperature_compensation(full_soc, 0.0);
        assert!(temp_comp_full <= 105.0, "100% SOC should not exceed 105%");

        // Aging compensation at 0% SOC
        let aging_comp_zero = compensate_aging(zero_soc, 5.0, 0.02);
        assert_eq!(aging_comp_zero, 0.0, "0% SOC should remain 0%");

        // Aging compensation at 100% SOC
        let aging_comp_full = compensate_aging(full_soc, 5.0, 0.02);
        assert!(aging_comp_full <= 100.0, "100% SOC should not exceed 100%");
    }

    #[test]
    fn test_temperature_compensation_fractional_values() {
        // Test with fractional SOC values
        let fractional_soc = 37.5;

        let result = default_temperature_compensation(fractional_soc, 10.0);
        assert!(result.is_finite(), "Result should be finite");
        assert!(result >= 0.0, "Result should be non-negative");
    }

    #[test]
    fn test_aging_compensation_fractional_values() {
        // Test with fractional age and factor values
        let result = compensate_aging(50.0, 2.5, 0.015);
        assert!(result.is_finite(), "Result should be finite");
        assert!(result >= 0.0, "Result should be non-negative");
        assert!(result < 50.0, "Result should be less than base SOC");
    }

    #[test]
    fn test_compensation_negative_coefficient() {
        // Test with negative temperature coefficient (unusual but possible)
        let result = compensate_temperature(50.0, 0.0, 25.0, -0.0005);
        assert!(result.is_finite(), "Result should be finite");
    }

    #[test]
    fn test_compensation_zero_coefficient() {
        // Test with zero coefficient (should result in no change)
        let result = compensate_temperature(50.0, 0.0, 25.0, 0.0);
        assert_eq!(result, 50.0, "Zero coefficient should result in no change");
    }

    #[test]
    fn test_aging_compensation_zero_factor() {
        // Test with zero aging factor (should result in no change)
        let result = compensate_aging(50.0, 5.0, 0.0);
        assert_eq!(result, 50.0, "Zero aging factor should result in no change");
    }

    #[test]
    fn test_temperature_compensation_precision() {
        // Test that compensation maintains reasonable precision
        let base_soc = 50.123456;
        let result = default_temperature_compensation(base_soc, 25.0);

        // At nominal temperature, should be very close to original
        assert!(
            (result - base_soc).abs() < 0.001,
            "Should maintain precision"
        );
    }

    #[test]
    fn test_combined_compensation_effects() {
        // Test that temperature and aging can be applied sequentially
        let base_soc = 50.0;

        // Apply temperature compensation
        let temp_comp = default_temperature_compensation(base_soc, 0.0);

        // Apply aging compensation to the result
        let final_comp = compensate_aging(temp_comp, 5.0, 0.02);

        // Both should have been applied
        assert!(final_comp.is_finite(), "Result should be finite");
        assert!(
            (0.0..=100.0).contains(&final_comp),
            "Result should be in valid range"
        );
    }
}
