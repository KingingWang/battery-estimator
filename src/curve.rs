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

        // Boundary checks
        if voltage_mv >= self.max_voltage_mv as i32 {
            return Ok(self.points[(self.len - 1) as usize].soc());
        }
        if voltage_mv <= self.min_voltage_mv as i32 {
            return Ok(self.points[0].soc());
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
        assert_eq!(curve.voltage_to_soc(2.9).unwrap(), 0.0); // 低于最小值
        assert_eq!(curve.voltage_to_soc(4.1).unwrap(), 100.0); // 高于最大值

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
}
