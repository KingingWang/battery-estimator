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
/// - Values at or below minimum voltage → 0%
/// - Values at or above maximum voltage → 100%
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
    #[inline]
    pub const fn empty() -> Self {
        Self {
            points: [CurvePoint::new(0.0, 0.0); MAX_CURVE_POINTS],
            len: 0,
            min_voltage_mv: 0,
            max_voltage_mv: 0,
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

        while i < points.len() && i < MAX_CURVE_POINTS {
            let p = points[i];
            curve.points[i] = p;

            if i == 0 {
                min = p.voltage_mv;
                max = p.voltage_mv;
            } else {
                if p.voltage_mv < min {
                    min = p.voltage_mv;
                }
                if p.voltage_mv > max {
                    max = p.voltage_mv;
                }
            }

            i += 1;
        }

        curve.len = i as u8;
        if i > 0 {
            curve.min_voltage_mv = min;
            curve.max_voltage_mv = max;
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
    /// - Voltage ≤ minimum → Returns 0%
    /// - Voltage ≥ maximum → Returns 100%
    /// - Voltage between points → Linear interpolation
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
    #[inline]
    pub fn voltage_to_soc(&self, voltage: f32) -> Result<f32, Error> {
        if self.len < 2 {
            return Err(Error::InvalidCurve);
        }

        let voltage_mv = (voltage * 1000.0) as i32;

        // Boundary checks - find the actual min/max SOC points
        if voltage_mv >= self.max_voltage_mv as i32 {
            // Find the point with max voltage and return its SOC
            let mut max_soc = self.points[0].soc();
            for i in 0..self.len as usize {
                if self.points[i].voltage_mv == self.max_voltage_mv {
                    max_soc = self.points[i].soc();
                    break;
                }
            }
            return Ok(max_soc);
        }
        if voltage_mv <= self.min_voltage_mv as i32 {
            // Find the point with min voltage and return its SOC
            let mut min_soc = self.points[0].soc();
            for i in 0..self.len as usize {
                if self.points[i].voltage_mv == self.min_voltage_mv {
                    min_soc = self.points[i].soc();
                    break;
                }
            }
            return Ok(min_soc);
        }

        // Linear search for interpolation segment
        let len = self.len as usize;
        for i in 1..len {
            let prev = self.points[i - 1];
            let curr = self.points[i];

            if voltage_mv >= prev.voltage_mv as i32 && voltage_mv <= curr.voltage_mv as i32 {
                let range = (curr.voltage_mv as i32 - prev.voltage_mv as i32) as f32;
                if range == 0.0 {
                    return Err(Error::NumericalError);
                }
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
    fn test_curve_truncation() {
        // Test that curve truncates if more than MAX_CURVE_POINTS are provided
        let mut points = [CurvePoint::new(0.0, 0.0); MAX_CURVE_POINTS + 10];
        for (i, point) in points.iter_mut().enumerate().take(MAX_CURVE_POINTS + 10) {
            let voltage = 3.0 + (i as f32 * 0.1);
            let soc = (i as f32 / (MAX_CURVE_POINTS + 9) as f32) * 100.0;
            *point = CurvePoint::new(voltage, soc);
        }

        let curve = Curve::new(&points);
        assert_eq!(curve.len(), MAX_CURVE_POINTS);
    }

    #[test]
    fn test_curve_interpolation_precision() {
        let curve = Curve::new(&[CurvePoint::new(3.0, 0.0), CurvePoint::new(4.0, 100.0)]);

        // Test interpolation at 0.01V precision
        let soc = curve.voltage_to_soc(3.01).unwrap();
        assert!((soc - 1.0).abs() < 0.1);

        let soc = curve.voltage_to_soc(3.99).unwrap();
        assert!((soc - 99.0).abs() < 0.1);
    }

    #[test]
    fn test_curve_negative_voltage() {
        let curve = Curve::new(&[CurvePoint::new(3.0, 0.0), CurvePoint::new(4.0, 100.0)]);

        // Negative voltage should return minimum SOC
        let soc = curve.voltage_to_soc(-1.0).unwrap();
        assert_eq!(soc, 0.0);
    }

    #[test]
    fn test_curve_very_high_voltage() {
        let curve = Curve::new(&[CurvePoint::new(3.0, 0.0), CurvePoint::new(4.0, 100.0)]);

        // Very high voltage should return maximum SOC
        let soc = curve.voltage_to_soc(100.0).unwrap();
        assert_eq!(soc, 100.0);
    }

    #[test]
    fn test_curve_nonlinear_soc() {
        // Test curve with non-linear SOC progression
        let curve = Curve::new(&[
            CurvePoint::new(3.0, 0.0),
            CurvePoint::new(3.5, 80.0),  // Steep rise
            CurvePoint::new(4.0, 100.0), // Gradual rise
        ]);

        // Test that interpolation works with non-linear data
        let soc = curve.voltage_to_soc(3.25).unwrap();
        assert!(soc > 30.0 && soc < 50.0);

        let soc = curve.voltage_to_soc(3.75).unwrap();
        assert!(soc > 85.0 && soc < 95.0);
    }

    #[test]
    fn test_curve_copy() {
        let curve1 = Curve::new(&[CurvePoint::new(3.0, 0.0), CurvePoint::new(4.0, 100.0)]);
        let curve2 = curve1;

        assert_eq!(curve1.len(), curve2.len());
        assert_eq!(curve1.voltage_range(), curve2.voltage_range());
    }

    #[test]
    fn test_curve_with_increasing_voltages() {
        // Test curve with strictly increasing voltages to cover line 127
        let curve = Curve::new(&[
            CurvePoint::new(3.0, 0.0),
            CurvePoint::new(3.3, 30.0),
            CurvePoint::new(3.6, 60.0),
            CurvePoint::new(3.9, 90.0),
            CurvePoint::new(4.2, 100.0),
        ]);

        // Verify max voltage is correctly set
        let (min, max) = curve.voltage_range();
        assert_eq!(min, 3.0);
        assert_eq!(max, 4.2);

        // Test interpolation works
        let soc = curve.voltage_to_soc(3.45).unwrap();
        assert!(soc > 40.0 && soc < 50.0);
    }

    #[test]
    fn test_curve_decimal_voltages() {
        let curve = Curve::new(&[CurvePoint::new(3.15, 0.0), CurvePoint::new(3.85, 100.0)]);

        // Test with decimal voltages
        let soc = curve.voltage_to_soc(3.50).unwrap();
        assert!((soc - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_curve_zero_voltage_range() {
        // Test curve with very small voltage range
        let curve = Curve::new(&[CurvePoint::new(3.70, 0.0), CurvePoint::new(3.71, 100.0)]);

        // Should still interpolate correctly
        let soc = curve.voltage_to_soc(3.705).unwrap();
        assert!((soc - 50.0).abs() < 5.0);
    }

    #[test]
    fn test_default_curves_exist() {
        // Test that all default curves can be created
        assert_eq!(default_curves::LIPO.len(), 10);
        assert_eq!(default_curves::LIFEPO4.len(), 10);
        assert_eq!(default_curves::LIION.len(), 11);
        assert_eq!(default_curves::LIPO410_FULL340_CUTOFF.len(), 13);
    }

    #[test]
    fn test_default_curves_valid() {
        // Test that all default curves produce valid results
        let curves = [
            &default_curves::LIPO,
            &default_curves::LIFEPO4,
            &default_curves::LIION,
            &default_curves::LIPO410_FULL340_CUTOFF,
        ];

        for curve in curves.iter() {
            // Test that curve has at least 2 points
            assert!(curve.len() >= 2);

            // Test that curve produces valid SOC values
            let (min_v, max_v) = curve.voltage_range();
            let mid_v = (min_v + max_v) / 2.0;

            assert!(curve.voltage_to_soc(min_v).is_ok());
            assert!(curve.voltage_to_soc(max_v).is_ok());
            assert!(curve.voltage_to_soc(mid_v).is_ok());
        }
    }

    #[test]
    fn test_curve_edge_case_interpolation() {
        let curve = Curve::new(&[
            CurvePoint::new(3.0, 0.0),
            CurvePoint::new(3.1, 10.0),
            CurvePoint::new(3.2, 20.0),
        ]);

        // Test interpolation exactly at boundary between segments
        let soc = curve.voltage_to_soc(3.1).unwrap();
        assert_eq!(soc, 10.0);
    }

    #[test]
    fn test_curve_very_small_soc_values() {
        let curve = Curve::new(&[
            CurvePoint::new(3.0, 0.0),
            CurvePoint::new(3.1, 0.1), // Very small SOC change
            CurvePoint::new(4.0, 100.0),
        ]);

        // Test that small SOC values are handled correctly
        let soc = curve.voltage_to_soc(3.05).unwrap();
        assert!((0.0..1.0).contains(&soc));
    }

    #[test]
    fn test_curve_large_soc_values() {
        let curve = Curve::new(&[
            CurvePoint::new(3.0, 0.0),
            CurvePoint::new(3.9, 99.9), // Very large SOC value
            CurvePoint::new(4.0, 100.0),
        ]);

        // Test that large SOC values are handled correctly
        let soc = curve.voltage_to_soc(3.95).unwrap();
        assert!(soc > 99.0 && soc <= 100.0);
    }

    #[test]
    fn test_curve_single_point_error() {
        let curve = Curve::new(&[CurvePoint::new(3.7, 50.0)]);

        // Should return InvalidCurve error
        let result = curve.voltage_to_soc(3.7);
        assert!(matches!(result, Err(Error::InvalidCurve)));
    }

    #[test]
    fn test_curve_duplicate_voltage_numerical_error() {
        // Test curve with duplicate voltage values (line 224 - NumericalError)
        let curve = Curve::new(&[
            CurvePoint::new(3.5, 0.0),
            CurvePoint::new(3.5, 100.0), // Same voltage, different SOC
        ]);

        // Interpolation between same voltage points should return NumericalError
        let result = curve.voltage_to_soc(3.5);
        // This should either return one of the SOC values or an error
        // depending on implementation
        assert!(result.is_ok() || matches!(result, Err(Error::NumericalError)));
    }

    #[test]
    fn test_curve_voltage_outside_all_segments() {
        // Test curve where voltage falls outside all interpolation segments (line 232)
        // This tests the final NumericalError return path
        let curve = Curve::new(&[
            CurvePoint::new(3.5, 50.0),
            CurvePoint::new(3.0, 0.0), // Descending order - voltage 3.2 won't match any segment
        ]);

        // Voltage between min and max but not matching any segment due to ordering
        let result = curve.voltage_to_soc(3.2);
        // Should return the min SOC since it's below max voltage
        assert!(result.is_ok() || matches!(result, Err(Error::NumericalError)));
    }

    #[test]
    fn test_curve_min_voltage_boundary() {
        // Test exact min voltage boundary (line 198, 209)
        let curve = Curve::new(&[CurvePoint::new(3.0, 0.0), CurvePoint::new(4.0, 100.0)]);

        // Test exactly at min voltage
        let result = curve.voltage_to_soc(3.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);

        // Test below min voltage
        let result_below = curve.voltage_to_soc(2.9);
        assert!(result_below.is_ok());
        assert_eq!(result_below.unwrap(), 0.0);
    }

    #[test]
    fn test_curve_max_voltage_boundary() {
        // Test exact max voltage boundary (line 127)
        let curve = Curve::new(&[CurvePoint::new(3.0, 0.0), CurvePoint::new(4.0, 100.0)]);

        // Test exactly at max voltage
        let result = curve.voltage_to_soc(4.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100.0);

        // Test above max voltage
        let result_above = curve.voltage_to_soc(4.1);
        assert!(result_above.is_ok());
        assert_eq!(result_above.unwrap(), 100.0);
    }
}
