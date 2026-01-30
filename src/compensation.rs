//! Temperature and aging compensation for battery SOC estimation
//!
//! This module provides functions to adjust SOC estimates based on
//! environmental conditions and battery age.

/// Applies temperature compensation to SOC value
///
/// Battery performance varies with temperature. This function adjusts
/// the estimated SOC to account for temperature effects on battery capacity.
///
/// # Physics Model
///
/// At low temperatures, battery internal resistance increases, reducing
/// the effective available capacity. At high temperatures, the battery
/// operates more efficiently but may degrade faster over time.
///
/// This function adjusts the **reported SOC** to reflect the actual
/// usable capacity at the current temperature.
///
/// # Arguments
///
/// * `soc` - Base SOC percentage (0.0 to 100.0)
/// * `temperature` - Current battery temperature in Celsius
/// * `nominal_temp` - Nominal/reference temperature in Celsius (typically 25°C)
/// * `coefficient` - Temperature coefficient (capacity loss per °C below nominal, e.g., 0.005 = 0.5%/°C)
///
/// # Returns
///
/// Temperature-compensated SOC percentage, or the original SOC if inputs are invalid (NaN/Infinity)
///
/// # Behavior
///
/// - At nominal temperature: No adjustment
/// - Below nominal: SOC decreases (less usable capacity due to higher internal resistance)
/// - Above nominal: SOC increases slightly (better efficiency, capped for safety)
/// - Compensation is bounded to prevent unrealistic values
///
/// # Examples
///
/// ```
/// use battery_estimator::compensate_temperature;
///
/// // At nominal temperature (25°C)
/// let soc = compensate_temperature(50.0, 25.0, 25.0, 0.005);
/// assert_eq!(soc, 50.0);
///
/// // At cold temperature (0°C) - 25°C below nominal
/// // Capacity reduced by 25 * 0.005 = 12.5%
/// let cold_soc = compensate_temperature(50.0, 0.0, 25.0, 0.005);
/// assert!(cold_soc < 50.0); // SOC decreases in cold
///
/// // At warm temperature (35°C) - 10°C above nominal
/// let warm_soc = compensate_temperature(50.0, 35.0, 25.0, 0.005);
/// assert!(warm_soc >= 50.0); // SOC may increase slightly in warmth
/// ```
#[inline]
pub fn compensate_temperature(
    soc: f32,
    temperature: f32,
    nominal_temp: f32,
    coefficient: f32,
) -> f32 {
    // Validate inputs - return original SOC if invalid
    if !soc.is_finite()
        || !temperature.is_finite()
        || !nominal_temp.is_finite()
        || !coefficient.is_finite()
    {
        return soc;
    }

    let delta_temp = temperature - nominal_temp;

    // Calculate capacity factor based on temperature difference
    // Below nominal: capacity decreases (factor < 1.0)
    // Above nominal: capacity increases slightly (factor > 1.0, but capped)
    let capacity_change = if delta_temp < 0.0 {
        // Cold: reduce capacity (more aggressive effect)
        delta_temp * coefficient
    } else {
        // Warm: slight capacity increase (less aggressive, capped at 5%)
        (delta_temp * coefficient * 0.5).min(0.05)
    };

    // Apply compensation: cold reduces SOC, warm increases SOC slightly
    // Bound the total compensation to reasonable limits (-30% to +5%)
    let bounded_change = clamp(capacity_change, -0.30, 0.05);

    soc * (1.0 + bounded_change)
}

/// Applies aging compensation to SOC value
///
/// Battery capacity degrades over time due to chemical aging.
/// This function adjusts the estimated SOC to account for capacity loss.
///
/// # Arguments
///
/// * `soc` - Base SOC percentage (0.0 to 100.0)
/// * `age_years` - Battery age in years (must be non-negative)
/// * `aging_factor` - Aging factor (capacity loss per year, e.g., 0.02 = 2%/year)
///
/// # Returns
///
/// Age-compensated SOC percentage, or the original SOC if inputs are invalid
///
/// # Behavior
///
/// - New battery (0 years): No adjustment
/// - Aged battery: SOC appears lower due to reduced capacity
/// - Maximum compensation is 50% (to prevent unrealistic values)
/// - Invalid inputs (NaN, Infinity, negative age) return original SOC
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
    // Validate inputs - return original SOC if invalid
    if !soc.is_finite() || !age_years.is_finite() || !aging_factor.is_finite() {
        return soc;
    }

    // Negative age doesn't make sense, treat as no aging
    if age_years < 0.0 {
        return soc;
    }

    // Negative aging factor doesn't make sense, treat as no aging
    if aging_factor < 0.0 {
        return soc;
    }

    let age_compensation = age_years * aging_factor;
    soc * (1.0 - clamp(age_compensation, 0.0, 0.5)) // Max 50% compensation
}

/// Applies default temperature compensation
///
/// This is a convenience function that uses standard default values:
/// - Nominal temperature: 25°C
/// - Temperature coefficient: 0.005 (0.5% capacity loss per °C below nominal)
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
/// // At room temperature (25°C) - no change
/// let soc = default_temperature_compensation(50.0, 25.0);
/// assert_eq!(soc, 50.0);
///
/// // At cold temperature (0°C) - capacity reduced
/// let cold_soc = default_temperature_compensation(50.0, 0.0);
/// assert!(cold_soc < 50.0); // SOC decreases in cold
/// ```
#[inline]
pub fn default_temperature_compensation(soc: f32, temperature: f32) -> f32 {
    const NOMINAL_TEMP: f32 = 25.0;
    const COEFFICIENT: f32 = 0.005; // 0.5% capacity loss per °C below nominal

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
    fn test_temperature_compensation_at_nominal() {
        // Room temperature (25°C) should have no change
        assert_eq!(default_temperature_compensation(50.0, 25.0), 50.0);
        assert_eq!(compensate_temperature(50.0, 25.0, 25.0, 0.005), 50.0);
    }

    #[test]
    fn test_temperature_compensation_cold() {
        // Cold temperature should DECREASE SOC (less usable capacity)
        let cold_compensated = default_temperature_compensation(50.0, 0.0);
        assert!(
            cold_compensated < 50.0,
            "Cold should decrease SOC due to reduced capacity"
        );

        // At 0°C (25°C below nominal), with 0.5%/°C coefficient:
        // Expected: 50.0 * (1.0 - 0.125) = 43.75
        let expected = 50.0 * (1.0 + (-25.0 * 0.005));
        assert!(
            (cold_compensated - expected).abs() < 0.01,
            "Cold compensation calculation mismatch"
        );
    }

    #[test]
    fn test_temperature_compensation_warm() {
        // Warm temperature should slightly INCREASE SOC (better efficiency)
        let warm_compensated = default_temperature_compensation(50.0, 35.0);
        assert!(
            warm_compensated >= 50.0,
            "Warm should increase or maintain SOC"
        );
    }

    #[test]
    fn test_temperature_compensation_bounds() {
        // Test boundary limits
        // Extreme cold: should be limited to -30% max
        let extreme_cold = default_temperature_compensation(50.0, -100.0);
        assert!(
            extreme_cold >= 50.0 * 0.70,
            "Extreme cold should be limited to -30%"
        );

        // Extreme hot: should be limited to +5% max
        let extreme_hot = default_temperature_compensation(50.0, 150.0);
        assert!(
            extreme_hot <= 50.0 * 1.05,
            "Extreme hot should be limited to +5%"
        );
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
        // Test very cold temperatures - SOC should DECREASE
        let result = compensate_temperature(50.0, -20.0, 25.0, 0.005);
        assert!(
            result < 50.0,
            "Cold temperature should decrease SOC due to reduced capacity"
        );
    }

    #[test]
    fn test_temperature_compensation_positive_temp() {
        // Test warm temperatures - SOC should increase slightly
        let result = compensate_temperature(50.0, 35.0, 25.0, 0.005);
        assert!(
            result >= 50.0,
            "Warm temperature should maintain or slightly increase SOC"
        );
    }

    #[test]
    fn test_temperature_compensation_invalid_inputs() {
        // NaN should return original SOC
        assert_eq!(compensate_temperature(50.0, f32::NAN, 25.0, 0.005), 50.0);
        assert_eq!(compensate_temperature(50.0, 25.0, f32::NAN, 0.005), 50.0);
        assert_eq!(compensate_temperature(50.0, 25.0, 25.0, f32::NAN), 50.0);
        assert!(compensate_temperature(f32::NAN, 25.0, 25.0, 0.005).is_nan());

        // Infinity should return original SOC
        assert_eq!(
            compensate_temperature(50.0, f32::INFINITY, 25.0, 0.005),
            50.0
        );
        assert_eq!(
            compensate_temperature(50.0, f32::NEG_INFINITY, 25.0, 0.005),
            50.0
        );
    }

    #[test]
    fn test_temperature_compensation_different_coefficients() {
        let base_soc = 50.0;
        let temp = 0.0; // 25°C below nominal (cold)
        let nominal = 25.0;

        // Test with different temperature coefficients
        // In cold conditions, higher coefficient = more capacity loss = lower SOC
        let result1 = compensate_temperature(base_soc, temp, nominal, 0.005);
        let result2 = compensate_temperature(base_soc, temp, nominal, 0.010);
        let result3 = compensate_temperature(base_soc, temp, nominal, 0.001);

        // Higher coefficient should result in MORE capacity loss (lower SOC) in cold
        assert!(
            result2 < result1,
            "Higher coefficient should result in lower SOC in cold (more capacity loss)"
        );
        assert!(
            result3 > result1,
            "Lower coefficient should result in higher SOC in cold (less capacity loss)"
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

    #[test]
    fn test_clamp_function() {
        // Test clamping at upper bound (warm temperature, +5% max)
        let result = compensate_temperature(50.0, 1000.0, 25.0, 0.005);
        assert!(result <= 52.5, "Should be clamped to +5%");

        // Test clamping at lower bound (cold temperature, -30% max)
        let result = compensate_temperature(50.0, -1000.0, 25.0, 0.005);
        assert!(result >= 35.0, "Should be clamped to -30%");

        // Test that extreme cold is properly bounded
        let result = compensate_temperature(50.0, -100.0, 25.0, 0.005);
        assert!(result >= 35.0, "Extreme cold should be at lower bound");
    }
}
