//! Voltage-SOC curve definitions and interpolation
//!
//! This module provides the [`Curve`] struct for representing battery
//! discharge curves and converting voltage measurements to state-of-charge (SOC) values.

use crate::{CurvePoint, Error};

/// Maximum number of points allowed in a voltage curve
///
/// This limit ensures predictable memory usage and prevents excessive
/// curve sizes that could impact performance in embedded systems.
pub const MAX_CURVE_POINTS: usize = 32;

/// A voltage-to-SOC curve for battery state-of-charge estimation
///
/// This struct represents a discharge curve that maps battery voltage
/// to state-of-charge percentage using linear interpolation between data points.
///
/// # Memory Optimization
///
/// The curve is stored using fixed-size arrays with optimized types:
/// - `points`: Fixed array of 32 points
/// - `len`: `u8` for point count (vs `usize`, saves memory)
/// - `min_voltage_mv`/`max_voltage_mv`: `u16` for voltage limits
/// - `min_soc_tenth`/`max_soc_tenth`: `u16` for cached SOC values (tenth of percent)
///
/// # Examples
///
/// ```no_run
/// use battery_estimator::{Curve, CurvePoint};
///
/// // Create a custom curve
/// const CUSTOM_CURVE: Curve = Curve::new(&[
///     CurvePoint::new(3.0, 0.0),
///     CurvePoint::new(3.5, 50.0),
///     CurvePoint::new(4.0, 100.0),
/// ]);
///
/// // Use the curve
/// match CUSTOM_CURVE.voltage_to_soc(3.75) {
///     Ok(soc) => println!("SOC: {:.1}%", soc),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
///
/// # Interpolation
///
/// The curve uses linear interpolation between points:
/// - Values at or below minimum voltage → Returns min SOC
/// - Values at or above maximum voltage → Returns max SOC
/// - Values between points → Linear interpolation
#[derive(Debug, Clone, Copy)]
pub struct Curve {
    /// Array of curve points (fixed size for memory efficiency)
    points: [CurvePoint; MAX_CURVE_POINTS],

    /// Number of points in the curve (0-255)
    len: u8,

    /// Minimum voltage in millivolts
    min_voltage_mv: u16,

    /// Maximum voltage in millivolts
    max_voltage_mv: u16,

    /// SOC at minimum voltage (cached in tenths of percent)
    min_soc_tenth: u16,

    /// SOC at maximum voltage (cached in tenths of percent)
    max_soc_tenth: u16,
}

impl Curve {
    /// Creates an empty curve with no points
    ///
    /// # Examples
    ///
    /// ```
    /// use battery_estimator::Curve;
    ///
    /// let empty = Curve::empty();
    /// assert!(empty.is_empty());
    /// assert_eq!(empty.len(), 0);
    /// ```
    pub const fn empty() -> Self {
        Self {
            points: [CurvePoint::new(0.0, 0.0); MAX_CURVE_POINTS],
            len: 0,
            min_voltage_mv: 0,
            max_voltage_mv: 0,
            min_soc_tenth: 0,
            max_soc_tenth: 0,
        }
    }

    /// Creates a new curve from a slice of points
    ///
    /// # Arguments
    ///
    /// * `points` - Slice of [`CurvePoint`] values, ordered by increasing voltage
    ///
    /// # Notes
    ///
    /// - Points must be ordered by increasing voltage
    /// - Maximum of 32 points will be stored
    /// - Minimum of 2 points required for valid interpolation
    ///
    /// # Examples
    ///
    /// ```
    /// use battery_estimator::{Curve, CurvePoint};
    ///
    /// let curve = Curve::new(&[
    ///     CurvePoint::new(3.0, 0.0),
    ///     CurvePoint::new(3.5, 50.0),
    ///     CurvePoint::new(4.0, 100.0),
    /// ]);
    /// ```
    pub const fn new(points: &[CurvePoint]) -> Self {
        let mut curve = Self::empty();
        let mut i = 0usize;

        let mut min = 0u16;
        let mut max = 0u16;
        let mut min_soc = 0u16;
        let mut max_soc = 0u16;

        while i < points.len() && i < MAX_CURVE_POINTS {
            let p = points[i];
            curve.points[i] = p;

            if i == 0 {
                // First point initializes all values
                min = p.voltage_mv;
                max = p.voltage_mv;
                min_soc = p.soc_tenth;
                max_soc = p.soc_tenth;
            } else {
                // Update min/max voltage and their corresponding SOC values
                if p.voltage_mv < min {
                    min = p.voltage_mv;
                    min_soc = p.soc_tenth;
                }
                if p.voltage_mv > max {
                    max = p.voltage_mv;
                    max_soc = p.soc_tenth;
                }
            }

            i += 1;
        }

        curve.len = i as u8;
        if i > 0 {
            curve.min_voltage_mv = min;
            curve.max_voltage_mv = max;
            curve.min_soc_tenth = min_soc;
            curve.max_soc_tenth = max_soc;
        }
        curve
    }

    /// Converts a voltage measurement to state-of-charge (SOC) percentage
    ///
    /// # Arguments
    ///
    /// * `voltage` - Battery voltage in volts
    ///
    /// # Returns
    ///
    /// * `Ok(soc)` - SOC percentage (0.0 to 100.0)
    /// * `Err(Error::InvalidCurve)` - Curve has fewer than 2 points
    /// * `Err(Error::NumericalError)` - Division by zero or calculation error
    ///
    /// # Behavior
    ///
    /// - Voltage ≤ minimum → Returns min SOC
    /// - Voltage ≥ maximum → Returns max SOC
    /// - Voltage between points → Linear interpolation
    ///
    /// # Performance
    ///
    /// This method uses binary search (via `partition_point`) for O(log n) lookup
    /// and cached SOC values for O(1) boundary checks.
    ///
    /// # Examples
    ///
    /// ```
    /// use battery_estimator::{Curve, CurvePoint};
    ///
    /// let curve = Curve::new(&[
    ///     CurvePoint::new(3.0, 0.0),
    ///     CurvePoint::new(3.5, 50.0),
    ///     CurvePoint::new(4.0, 100.0),
    /// ]);
    ///
    /// // At minimum voltage
    /// assert_eq!(curve.voltage_to_soc(3.0).unwrap(), 0.0);
    ///
    /// // At maximum voltage
    /// assert_eq!(curve.voltage_to_soc(4.0).unwrap(), 100.0);
    ///
    /// // Midpoint interpolation
    /// assert_eq!(curve.voltage_to_soc(3.5).unwrap(), 50.0);
    /// ```
    pub fn voltage_to_soc(&self, voltage: f32) -> Result<f32, Error> {
        if self.len < 2 {
            return Err(Error::InvalidCurve);
        }

        let voltage_mv = (voltage * 1000.0) as i32;

        // Cache frequently used values to avoid repeated conversions
        let max_voltage_mv = self.max_voltage_mv as i32;
        let min_voltage_mv = self.min_voltage_mv as i32;
        let max_soc = self.max_soc_tenth as f32 / 10.0;
        let min_soc = self.min_soc_tenth as f32 / 10.0;

        // Boundary checks - use cached SOC values for O(1) lookup
        if voltage_mv >= max_voltage_mv {
            return Ok(max_soc);
        }
        if voltage_mv <= min_voltage_mv {
            return Ok(min_soc);
        }

        // Binary search for interpolation segment using Rust's partition_point
        let points = &self.points[..self.len as usize];

        // Find the index of the first point with voltage > target voltage
        let idx = points.partition_point(|p| p.voltage_mv as i32 <= voltage_mv);

        // Check if we found a valid interpolation segment
        if idx > 0 && idx < points.len() {
            let prev = points[idx - 1];
            let curr = points[idx];

            if voltage_mv >= prev.voltage_mv as i32 && voltage_mv <= curr.voltage_mv as i32 {
                let range = (curr.voltage_mv as i32 - prev.voltage_mv as i32) as f32;
                let ratio = (voltage_mv - prev.voltage_mv as i32) as f32 / range;
                let soc = prev.soc() + ratio * (curr.soc() - prev.soc());
                return Ok(soc);
            }
        }

        Err(Error::NumericalError)
    }

    /// Returns the voltage range of the curve
    ///
    /// # Returns
    ///
    /// Tuple of (minimum_voltage, maximum_voltage) in volts
    ///
    /// # Examples
    ///
    /// ```
    /// use battery_estimator::{Curve, CurvePoint};
    ///
    /// let curve = Curve::new(&[
    ///     CurvePoint::new(3.0, 0.0),
    ///     CurvePoint::new(4.0, 100.0),
    /// ]);
    ///
    /// let (min, max) = curve.voltage_range();
    /// assert_eq!(min, 3.0);
    /// assert_eq!(max, 4.0);
    /// ```
    #[inline]
    pub const fn voltage_range(&self) -> (f32, f32) {
        (
            self.min_voltage_mv as f32 / 1000.0,
            self.max_voltage_mv as f32 / 1000.0,
        )
    }

    /// Returns the number of points in the curve
    ///
    /// # Examples
    ///
    /// ```
    /// use battery_estimator::{Curve, CurvePoint};
    ///
    /// let curve = Curve::new(&[
    ///     CurvePoint::new(3.0, 0.0),
    ///     CurvePoint::new(3.5, 50.0),
    ///     CurvePoint::new(4.0, 100.0),
    /// ]);
    ///
    /// assert_eq!(curve.len(), 3);
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns `true` if the curve has no points
    ///
    /// # Examples
    ///
    /// ```
    /// use battery_estimator::Curve;
    ///
    /// let empty = Curve::empty();
    /// assert!(empty.is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Predefined battery voltage curves
///
/// This module contains built-in voltage curves for common battery types.
/// These curves are optimized for typical discharge characteristics.
pub mod default_curves {
    use super::*;

    /// Standard Lithium Polymer (LiPo) battery curve
    ///
    /// - Full charge: 4.2V
    /// - Cutoff: 3.2V
    /// - Nominal: 3.7V
    /// - Points: 10
    pub const LIPO: Curve = Curve::new(&[
        CurvePoint::new(3.20, 0.0),
        CurvePoint::new(3.30, 5.0),
        CurvePoint::new(3.40, 10.0),
        CurvePoint::new(3.50, 20.0),
        CurvePoint::new(3.60, 30.0),
        CurvePoint::new(3.70, 50.0),
        CurvePoint::new(3.80, 70.0),
        CurvePoint::new(3.90, 85.0),
        CurvePoint::new(4.00, 95.0),
        CurvePoint::new(4.20, 100.0),
    ]);

    /// Lithium Iron Phosphate (LiFePO4) battery curve
    ///
    /// - Full charge: 3.65V
    /// - Cutoff: 3.0V
    /// - Nominal: 3.2V
    /// - Points: 10
    /// - Features: Very flat discharge curve, long cycle life
    pub const LIFEPO4: Curve = Curve::new(&[
        CurvePoint::new(2.50, 0.0),
        CurvePoint::new(2.80, 15.0),
        CurvePoint::new(3.00, 35.0),
        CurvePoint::new(3.10, 45.0),
        CurvePoint::new(3.20, 55.0),
        CurvePoint::new(3.30, 65.0),
        CurvePoint::new(3.40, 75.0),
        CurvePoint::new(3.50, 85.0),
        CurvePoint::new(3.60, 95.0),
        CurvePoint::new(3.65, 100.0),
    ]);

    /// Standard Lithium Ion (Li-Ion) battery curve
    ///
    /// - Full charge: 4.2V
    /// - Cutoff: 3.3V
    /// - Nominal: 3.7V
    /// - Points: 11
    pub const LIION: Curve = Curve::new(&[
        CurvePoint::new(2.50, 0.0),
        CurvePoint::new(3.00, 30.0),
        CurvePoint::new(3.30, 50.0),
        CurvePoint::new(3.50, 65.0),
        CurvePoint::new(3.60, 70.0),
        CurvePoint::new(3.70, 75.0),
        CurvePoint::new(3.80, 80.0),
        CurvePoint::new(3.90, 85.0),
        CurvePoint::new(4.00, 90.0),
        CurvePoint::new(4.10, 95.0),
        CurvePoint::new(4.20, 100.0),
    ]);

    /// Conservative LiPo curve for extended battery life
    ///
    /// - Full charge: 4.1V (lower than standard 4.2V)
    /// - Cutoff: 3.4V (higher than standard 3.2V)
    /// - Nominal: 3.77V
    /// - Points: 13
    ///
    /// # Use Case
    ///
    /// This curve prioritizes battery longevity over maximum capacity:
    /// - **Lower charge voltage** (4.1V) reduces charging stress
    /// - **Higher cutoff** (3.4V) prevents deep discharge
    /// - **Trade-off**: ~15-20% less usable capacity for ~30% longer cycle life
    ///
    /// # When to Use
    ///
    /// - Applications where battery replacement is difficult
    /// - Devices requiring very long service life
    /// - Systems prioritizing reliability over runtime
    pub const LIPO410_FULL340_CUTOFF: Curve = Curve::new(&[
        CurvePoint::new(3.40, 0.0),
        CurvePoint::new(3.48, 5.0),
        CurvePoint::new(3.53, 10.0),
        CurvePoint::new(3.62, 20.0),
        CurvePoint::new(3.68, 30.0),
        CurvePoint::new(3.73, 40.0),
        CurvePoint::new(3.77, 50.0),
        CurvePoint::new(3.81, 60.0),
        CurvePoint::new(3.85, 70.0),
        CurvePoint::new(3.90, 80.0),
        CurvePoint::new(3.97, 90.0),
        CurvePoint::new(4.03, 95.0),
        CurvePoint::new(4.10, 100.0),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curve_basic() {
        let curve = Curve::new(&[CurvePoint::new(3.0, 0.0), CurvePoint::new(4.0, 100.0)]);

        assert_eq!(curve.voltage_to_soc(3.0).unwrap(), 0.0);
        assert_eq!(curve.voltage_to_soc(4.0).unwrap(), 100.0);
        assert_eq!(curve.voltage_to_soc(3.5).unwrap(), 50.0);
    }

    #[test]
    fn test_curve_boundaries() {
        let curve = Curve::new(&[
            CurvePoint::new(3.0, 0.0),
            CurvePoint::new(3.5, 50.0),
            CurvePoint::new(4.0, 100.0),
        ]);

        // Test boundaries
        assert_eq!(curve.voltage_to_soc(2.9).unwrap(), 0.0);
        assert_eq!(curve.voltage_to_soc(4.1).unwrap(), 100.0);

        // Test intermediate values
        assert_eq!(curve.voltage_to_soc(3.25).unwrap(), 25.0);
        assert_eq!(curve.voltage_to_soc(3.75).unwrap(), 75.0);
    }

    #[test]
    fn test_curve_invalid() {
        let curve = Curve::new(&[CurvePoint::new(3.0, 0.0)]);

        // Curve with only one point should error
        assert!(curve.voltage_to_soc(3.5).is_err());
    }

    #[test]
    fn test_curve_empty() {
        let curve = Curve::empty();
        assert!(curve.is_empty());
        assert_eq!(curve.len(), 0);
        assert!(curve.voltage_to_soc(3.0).is_err());
    }

    #[test]
    fn test_curve_multiple_points() {
        let curve = Curve::new(&[
            CurvePoint::new(3.0, 0.0),
            CurvePoint::new(3.5, 50.0),
            CurvePoint::new(4.0, 100.0),
        ]);

        assert_eq!(curve.len(), 3);

        // Test exact points
        assert_eq!(curve.voltage_to_soc(3.0).unwrap(), 0.0);
        assert_eq!(curve.voltage_to_soc(3.5).unwrap(), 50.0);
        assert_eq!(curve.voltage_to_soc(4.0).unwrap(), 100.0);

        // Test interpolation
        let soc = curve.voltage_to_soc(3.25).unwrap();
        assert!((soc - 25.0).abs() < 0.1);

        let soc = curve.voltage_to_soc(3.75).unwrap();
        assert!((soc - 75.0).abs() < 0.1);
    }

    #[test]
    fn test_curve_voltage_range() {
        let curve = Curve::new(&[CurvePoint::new(3.0, 0.0), CurvePoint::new(4.0, 100.0)]);

        let (min, max) = curve.voltage_range();
        assert_eq!(min, 3.0);
        assert_eq!(max, 4.0);
    }

    #[test]
    fn test_curve_max_points() {
        // Test that curve handles maximum number of points
        let mut points = [CurvePoint::new(0.0, 0.0); MAX_CURVE_POINTS];
        for (i, point) in points.iter_mut().enumerate().take(MAX_CURVE_POINTS) {
            let voltage = 3.0 + (i as f32 * 0.1);
            let soc = (i as f32 / (MAX_CURVE_POINTS - 1) as f32) * 100.0;
            *point = CurvePoint::new(voltage, soc);
        }

        let curve = Curve::new(&points);
        assert_eq!(curve.len(), MAX_CURVE_POINTS);

        // Test interpolation at various points
        assert!(curve.voltage_to_soc(3.5).is_ok());
    }

    #[test]
    fn test_curve_numerical_error_fallback() {
        // Test the fallback NumericalError path when voltage is not found in any segment
        // This can happen with non-monotonic/decreasing voltage curves
        // The curve stores points in order, but with decreasing voltages
        // so the linear search won't find a valid segment
        let curve = Curve::new(&[
            CurvePoint::new(3.0, 0.0),
            CurvePoint::new(2.5, 50.0), // Decreasing voltage
            CurvePoint::new(2.0, 100.0),
        ]);

        // Voltage 2.7 is between 3.0 and 2.5 but not in increasing order
        // This should trigger NumericalError
        assert!(matches!(
            curve.voltage_to_soc(2.7),
            Err(Error::NumericalError)
        ));
    }

    #[test]
    fn test_curve_cached_soc_values() {
        // Test that cached SOC values are correctly computed
        let curve = Curve::new(&[
            CurvePoint::new(3.0, 5.0), // Non-zero min SOC
            CurvePoint::new(3.5, 50.0),
            CurvePoint::new(4.0, 95.0), // Non-100 max SOC
        ]);

        // At min voltage, should return cached min SOC (5.0%)
        assert_eq!(curve.voltage_to_soc(3.0).unwrap(), 5.0);

        // Below min voltage, should still return cached min SOC
        assert_eq!(curve.voltage_to_soc(2.5).unwrap(), 5.0);

        // At max voltage, should return cached max SOC (95.0%)
        assert_eq!(curve.voltage_to_soc(4.0).unwrap(), 95.0);

        // Above max voltage, should still return cached max SOC
        assert_eq!(curve.voltage_to_soc(4.5).unwrap(), 95.0);
    }

    #[test]
    fn test_curve_interpolation_precision() {
        let curve = Curve::new(&[
            CurvePoint::new(3.0, 0.0),
            CurvePoint::new(3.1, 10.0),
            CurvePoint::new(3.2, 20.0),
            CurvePoint::new(3.3, 30.0),
        ]);

        // Test precise interpolation
        assert_eq!(curve.voltage_to_soc(3.05).unwrap(), 5.0);
        assert_eq!(curve.voltage_to_soc(3.15).unwrap(), 15.0);
        assert_eq!(curve.voltage_to_soc(3.25).unwrap(), 25.0);
    }

    #[test]
    fn test_curve_single_segment() {
        let curve = Curve::new(&[CurvePoint::new(3.0, 0.0), CurvePoint::new(4.0, 100.0)]);

        // Single segment interpolation
        assert_eq!(curve.voltage_to_soc(3.25).unwrap(), 25.0);
        assert_eq!(curve.voltage_to_soc(3.5).unwrap(), 50.0);
        assert_eq!(curve.voltage_to_soc(3.75).unwrap(), 75.0);
    }

    #[test]
    fn test_curve_dense_points() {
        // Test with many closely spaced points - use array for no_std compatibility
        let points: [CurvePoint; 21] =
            core::array::from_fn(|i| CurvePoint::new(3.0 + i as f32 * 0.05, i as f32 * 5.0));

        let curve = Curve::new(&points);

        // Test that interpolation works with dense points
        for i in 0..20 {
            let voltage = 3.0 + i as f32 * 0.05 + 0.025;
            let expected_soc = i as f32 * 5.0 + 2.5;
            assert!((curve.voltage_to_soc(voltage).unwrap() - expected_soc).abs() < 0.01);
        }
    }

    #[test]
    fn test_curve_non_linear() {
        // Test with non-linear curve (exponential-like)
        let curve = Curve::new(&[
            CurvePoint::new(3.0, 0.0),
            CurvePoint::new(3.5, 20.0),
            CurvePoint::new(4.0, 60.0),
            CurvePoint::new(4.2, 100.0),
        ]);

        // Verify non-linear interpolation
        let soc_35 = curve.voltage_to_soc(3.5).unwrap();
        let soc_38 = curve.voltage_to_soc(3.8).unwrap();
        let soc_41 = curve.voltage_to_soc(4.1).unwrap();

        assert_eq!(soc_35, 20.0);
        // 3.8V is between 3.5V (20%) and 4.0V (60%)
        // ratio = (3.8 - 3.5) / (4.0 - 3.5) = 0.6
        // soc = 20 + 0.6 * 40 = 44.0
        assert!((soc_38 - 44.0).abs() < 0.1);
        // 4.1V is between 4.0V (60%) and 4.2V (100%)
        // ratio = (4.1 - 4.0) / (4.2 - 4.0) = 0.5
        // soc = 60 + 0.5 * 40 = 80.0
        assert!((soc_41 - 80.0).abs() < 0.1);
    }
}
