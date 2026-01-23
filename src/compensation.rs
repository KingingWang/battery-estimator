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
}
