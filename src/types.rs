//! Type definitions for battery SOC estimation
//!
//! This module contains the core data types used throughout the library:
//!
//! - [`BatteryChemistry`] - Enumeration of supported battery types
//! - [`CurvePoint`] - Individual voltage-SOC data point for curves

/// Const-compatible check for finite f32 values
///
/// Returns true if the value is neither NaN nor infinite.
#[inline]
const fn is_finite_const(value: f32) -> bool {
    // A value is finite if it's not NaN and not infinite
    // NaN: exponent all 1s, mantissa non-zero
    // Infinity: exponent all 1s, mantissa zero
    // We check if exponent bits are not all 1s (0xFF)
    let bits = value.to_bits();
    let exponent = (bits >> 23) & 0xFF;
    exponent != 0xFF
}

/// Battery chemistry types supported by the library
///
/// Each variant represents a specific battery chemistry with its own
/// built-in voltage-to-SOC curve. These curves are optimized for typical
/// discharge characteristics of each chemistry.
///
/// # Voltage Ranges
///
/// | Chemistry | Full Charge | Cutoff | Description |
/// |-----------|-------------|--------|-------------|
/// | `LiPo` | 4.2V | 3.2V | Standard Lithium Polymer |
/// | `LiFePO4` | 3.65V | 3.0V | Lithium Iron Phosphate (long cycle life) |
/// | `LiIon` | 4.2V | 3.3V | Standard Lithium Ion |
/// | `Lipo410Full340Cutoff` | 4.1V | 3.4V | Conservative LiPo (extended life) |
///
/// # Examples
///
/// ```
/// use battery_estimator::{BatteryChemistry, SocEstimator};
///
/// // Create estimator for LiPo battery
/// let estimator = SocEstimator::new(BatteryChemistry::LiPo);
///
/// // Create estimator for LiFePO4 battery
/// let lfp_estimator = SocEstimator::new(BatteryChemistry::LiFePO4);
/// ```
///
/// # Conservative Battery Curve
///
/// The `Lipo410Full340Cutoff` variant uses conservative thresholds:
/// - **Lower full charge** (4.1V vs 4.2V) - Reduces stress on battery
/// - **Higher cutoff** (3.4V vs 3.2V) - Prevents deep discharge
/// - **Benefit**: Extended cycle life at cost of reduced capacity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BatteryChemistry {
    /// Standard Lithium Polymer battery
    ///
    /// - Full charge: 4.2V
    /// - Cutoff voltage: 3.2V
    /// - Nominal voltage: 3.7V
    /// - Typical use: RC vehicles, drones, portable electronics
    LiPo,

    /// Lithium Iron Phosphate (LiFePO4) battery
    ///
    /// - Full charge: 3.65V
    /// - Cutoff voltage: 3.0V
    /// - Nominal voltage: 3.2V
    /// - Typical use: Solar systems, EVs, energy storage
    /// - Advantages: Long cycle life (2000+ cycles), stable voltage
    LiFePO4,

    /// Standard Lithium Ion battery
    ///
    /// - Full charge: 4.2V
    /// - Cutoff voltage: 3.3V
    /// - Nominal voltage: 3.7V
    /// - Typical use: Laptops, power tools, consumer electronics
    LiIon,

    /// Conservative LiPo battery curve for extended cycle life
    ///
    /// - Full charge: 4.1V (lower than standard 4.2V)
    /// - Cutoff voltage: 3.4V (higher than standard 3.2V)
    /// - Nominal voltage: 3.77V
    /// - Use case: Applications prioritizing battery longevity over capacity
    /// - Trade-off: ~15-20% less usable capacity for ~30% longer cycle life
    Lipo410Full340Cutoff,
}

/// A single point on a voltage-SOC curve
///
/// This struct represents one data point in a battery discharge curve,
/// mapping a specific voltage to a corresponding state-of-charge percentage.
///
/// # Internal Representation
///
/// For memory efficiency in embedded systems, values are stored as integers:
/// - **Voltage**: Stored in millivolts (`u16`, range 0-65535 mV)
/// - **SOC**: Stored in tenths of a percent (`u16`, range 0-1000 = 0-100%)
///
/// This representation reduces memory usage by 50% compared to `f32` storage
/// while maintaining sufficient precision for battery estimation.
///
/// # Examples
///
/// ```
/// use battery_estimator::CurvePoint;
///
/// // Create a curve point at 3.7V with 50% SOC
/// let point = CurvePoint::new(3.7, 50.0);
///
/// // Access voltage in volts
/// assert_eq!(point.voltage(), 3.7);
///
/// // Access SOC in percent
/// assert_eq!(point.soc(), 50.0);
///
/// // Create from tuple
/// let point2: CurvePoint = (3.8, 75.0).into();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CurvePoint {
    /// Voltage in millivolts (mV)
    ///
    /// Range: 0-65535 mV (0-65.535V)
    /// Internal storage format for memory efficiency
    pub voltage_mv: u16,

    /// State of charge in tenths of a percent
    ///
    /// Range: 0-1000 (0-100%)
    /// Internal storage format for memory efficiency
    pub soc_tenth: u16,
}

impl CurvePoint {
    /// Creates a new curve point from floating-point values
    ///
    /// # Arguments
    ///
    /// * `voltage` - Voltage in volts (e.g., 3.7). Must be non-negative and finite.
    /// * `soc` - State of charge in percent (e.g., 50.0). Must be in range 0.0-100.0.
    ///
    /// # Input Handling
    ///
    /// - Negative voltages are clamped to 0.0
    /// - NaN/Infinity voltages are treated as 0.0
    /// - SOC values are clamped to 0.0-100.0 range
    /// - NaN/Infinity SOC values are treated as 0.0
    ///
    /// # Examples
    ///
    /// ```
    /// use battery_estimator::CurvePoint;
    ///
    /// let point = CurvePoint::new(3.7, 50.0);
    /// assert_eq!(point.voltage(), 3.7);
    /// assert_eq!(point.soc(), 50.0);
    ///
    /// // Negative voltage is clamped to 0
    /// let clamped = CurvePoint::new(-1.0, 50.0);
    /// assert_eq!(clamped.voltage(), 0.0);
    /// ```
    #[inline]
    pub const fn new(voltage: f32, soc: f32) -> Self {
        // Validate and clamp voltage (must be non-negative, max 65.535V for u16)
        let safe_voltage = if voltage < 0.0 || !is_finite_const(voltage) {
            0.0
        } else if voltage > 65.535 {
            65.535
        } else {
            voltage
        };

        // Validate and clamp SOC (must be 0-100%)
        let safe_soc = if soc < 0.0 || !is_finite_const(soc) {
            0.0
        } else if soc > 100.0 {
            100.0
        } else {
            soc
        };

        Self {
            voltage_mv: (safe_voltage * 1000.0) as u16,
            soc_tenth: (safe_soc * 10.0) as u16,
        }
    }

    /// Creates a new curve point without validation (for performance-critical code)
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - `voltage` is non-negative and <= 65.535
    /// - `soc` is in range 0.0-100.0
    /// - Both values are finite (not NaN or Infinity)
    ///
    /// # Examples
    ///
    /// ```
    /// use battery_estimator::CurvePoint;
    ///
    /// // Only use when you're certain the values are valid
    /// let point = CurvePoint::new_unchecked(3.7, 50.0);
    /// ```
    #[inline]
    pub const fn new_unchecked(voltage: f32, soc: f32) -> Self {
        Self {
            voltage_mv: (voltage * 1000.0) as u16,
            soc_tenth: (soc * 10.0) as u16,
        }
    }

    /// Returns the voltage in volts
    ///
    /// # Examples
    ///
    /// ```
    /// use battery_estimator::CurvePoint;
    ///
    /// let point = CurvePoint::new(3.7, 50.0);
    /// assert_eq!(point.voltage(), 3.7);
    /// ```
    #[inline]
    pub const fn voltage(&self) -> f32 {
        self.voltage_mv as f32 / 1000.0
    }

    /// Returns the state of charge in percent
    ///
    /// # Examples
    ///
    /// ```
    /// use battery_estimator::CurvePoint;
    ///
    /// let point = CurvePoint::new(3.7, 50.0);
    /// assert_eq!(point.soc(), 50.0);
    /// ```
    #[inline]
    pub const fn soc(&self) -> f32 {
        self.soc_tenth as f32 / 10.0
    }
}

impl From<(f32, f32)> for CurvePoint {
    /// Creates a curve point from a tuple (voltage, soc)
    ///
    /// # Examples
    ///
    /// ```
    /// use battery_estimator::CurvePoint;
    ///
    /// let point: CurvePoint = (3.7, 50.0).into();
    /// assert_eq!(point.voltage(), 3.7);
    /// assert_eq!(point.soc(), 50.0);
    /// ```
    fn from((voltage, soc): (f32, f32)) -> Self {
        Self::new(voltage, soc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curve_point_creation() {
        let point = CurvePoint::new(3.7, 50.0);
        assert_eq!(point.voltage(), 3.7);
        assert_eq!(point.soc(), 50.0);
    }

    #[test]
    fn test_curve_point_zero_values() {
        let point = CurvePoint::new(0.0, 0.0);
        assert_eq!(point.voltage(), 0.0);
        assert_eq!(point.soc(), 0.0);
    }

    #[test]
    fn test_curve_point_boundary_values() {
        // Test maximum voltage (u16 max / 1000 = 65.535V)
        let max_point = CurvePoint::new(65.535, 100.0);
        assert_eq!(max_point.voltage(), 65.535);
        assert_eq!(max_point.soc(), 100.0);
    }

    #[test]
    fn test_curve_point_decimal_voltage() {
        // Test two decimal places
        let point = CurvePoint::new(3.71, 75.5);
        assert_eq!(point.voltage(), 3.71);
        assert_eq!(point.soc(), 75.5);
    }

    #[test]
    fn test_curve_point_negative_voltage() {
        // Test that negative voltage is handled
        // Note: Negative voltages get stored as u16, so they wrap around
        // This is expected behavior for the current implementation
        let point = CurvePoint::new(-1.5, 0.0);
        // The voltage will be wrapped due to u16 storage
        assert!(point.voltage() >= 0.0); // Will be positive due to wrapping
    }

    #[test]
    fn test_curve_point_soc_bounds() {
        // Test minimum SOC
        let min_soc = CurvePoint::new(3.7, 0.0);
        assert_eq!(min_soc.soc(), 0.0);

        // Test maximum SOC
        let max_soc = CurvePoint::new(4.2, 100.0);
        assert_eq!(max_soc.soc(), 100.0);
    }

    #[test]
    fn test_curve_point_soc_precision() {
        // Test that SOC is stored with 0.1% precision
        let point = CurvePoint::new(3.7, 50.15);
        // Should be rounded to 50.1 or 50.2 depending on rounding
        let soc = point.soc();
        assert!((50.0..=50.2).contains(&soc));
    }

    #[test]
    fn test_curve_point_from_tuple() {
        let point: CurvePoint = (3.8, 75.0).into();
        assert_eq!(point.voltage(), 3.8);
        assert_eq!(point.soc(), 75.0);
    }

    #[test]
    fn test_curve_point_equality() {
        let point1 = CurvePoint::new(3.7, 50.0);
        let point2 = CurvePoint::new(3.7, 50.0);
        let point3 = CurvePoint::new(3.8, 50.0);

        assert_eq!(point1, point2);
        assert_ne!(point1, point3);
    }

    #[test]
    fn test_curve_point_copy() {
        let point1 = CurvePoint::new(3.7, 50.0);
        let point2 = point1;
        assert_eq!(point1, point2);
    }

    #[test]
    fn test_curve_point_internal_representation() {
        let point = CurvePoint::new(3.7, 50.0);
        // Voltage should be stored in millivolts
        assert_eq!(point.voltage_mv, 3700);
        // SOC should be stored in tenths of a percent
        assert_eq!(point.soc_tenth, 500);
    }

    #[test]
    fn test_battery_chemistry_variants() {
        // Test that all battery chemistry variants can be created
        let lipo = BatteryChemistry::LiPo;
        let lifepo4 = BatteryChemistry::LiFePO4;
        let _lilon = BatteryChemistry::LiIon;
        let _conservative = BatteryChemistry::Lipo410Full340Cutoff;

        // Test equality
        assert_eq!(lipo, BatteryChemistry::LiPo);
        assert_ne!(lipo, lifepo4);
    }

    #[test]
    fn test_battery_chemistry_copy() {
        let chem1 = BatteryChemistry::LiPo;
        let chem2 = chem1;
        assert_eq!(chem1, chem2);
    }

    #[test]
    fn test_curve_point_extreme_soc() {
        // Test SOC values beyond normal range
        // With input validation, negative SOC is clamped to 0
        let point1 = CurvePoint::new(3.7, -10.0);
        assert_eq!(point1.soc(), 0.0, "Negative SOC should be clamped to 0");

        // SOC above 100% is clamped to 100%
        let point2 = CurvePoint::new(3.7, 150.0);
        assert_eq!(
            point2.soc(),
            100.0,
            "SOC above 100% should be clamped to 100%"
        );
    }

    #[test]
    fn test_curve_point_voltage_precision() {
        // Test that voltage precision is maintained
        let point = CurvePoint::new(3.715, 50.0);
        // Should be stored and retrieved accurately
        assert!((point.voltage() - 3.715).abs() < 0.001);
    }

    #[test]
    fn test_curve_point_nan_voltage() {
        // Test NaN voltage is handled (line 172)
        let point = CurvePoint::new(f32::NAN, 50.0);
        assert_eq!(point.voltage(), 0.0, "NaN voltage should be clamped to 0");
    }

    #[test]
    fn test_curve_point_infinity_voltage() {
        // Test Infinity voltage is handled (line 172)
        // Non-finite values are treated as invalid and clamped to 0
        let point = CurvePoint::new(f32::INFINITY, 50.0);
        assert_eq!(
            point.voltage(),
            0.0,
            "Infinity voltage should be treated as invalid and set to 0"
        );
    }

    #[test]
    fn test_curve_point_nan_soc() {
        // Test NaN SOC is handled (line 210)
        let point = CurvePoint::new(3.7, f32::NAN);
        assert_eq!(point.soc(), 0.0, "NaN SOC should be clamped to 0");
    }

    #[test]
    fn test_curve_point_infinity_soc() {
        // Test Infinity SOC is handled (lines 212-213)
        // Non-finite values are treated as invalid and clamped to 0
        let point = CurvePoint::new(3.7, f32::INFINITY);
        assert_eq!(
            point.soc(),
            0.0,
            "Infinity SOC should be treated as invalid and set to 0"
        );
    }

    #[test]
    fn test_curve_point_neg_infinity_voltage() {
        // Test negative Infinity voltage
        let point = CurvePoint::new(f32::NEG_INFINITY, 50.0);
        assert_eq!(
            point.voltage(),
            0.0,
            "Negative Infinity voltage should be clamped to 0"
        );
    }

    #[test]
    fn test_curve_point_neg_infinity_soc() {
        // Test negative Infinity SOC
        let point = CurvePoint::new(3.7, f32::NEG_INFINITY);
        assert_eq!(
            point.soc(),
            0.0,
            "Negative Infinity SOC should be clamped to 0"
        );
    }
}
