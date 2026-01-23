//! Error types for battery SOC estimation
//!
//! This module defines the error types that can occur during battery
//! state-of-charge estimation operations.

use core::fmt;

/// Errors that can occur during battery SOC estimation
///
/// This enum represents all possible error conditions that may arise
/// when using the battery estimator library.
///
/// # Examples
///
/// ```no_run
/// use battery_estimator::{BatteryChemistry, SocEstimator, Error};
///
/// let estimator = SocEstimator::new(BatteryChemistry::LiPo);
///
/// match estimator.estimate_soc(3.7) {
///     Ok(soc) => println!("SOC: {:.1}%", soc),
///     Err(Error::InvalidCurve) => eprintln!("Invalid battery curve"),
///     Err(Error::NumericalError) => eprintln!("Calculation error"),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    /// Voltage value is outside the valid range for the battery curve
    ///
    /// This error occurs when the input voltage is either:
    /// - Below the minimum voltage of the curve
    /// - Above the maximum voltage of the curve
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use battery_estimator::{BatteryChemistry, SocEstimator, Error};
    ///
    /// let estimator = SocEstimator::new(BatteryChemistry::LiPo);
    ///
    /// // Note: The estimator actually clamps values to the curve range,
    /// // so this error is typically not encountered in normal use
    /// ```
    VoltageOutOfRange,

    /// The voltage curve data is invalid
    ///
    /// This error occurs when:
    /// - The curve has fewer than 2 points (cannot interpolate)
    /// - The curve points are not properly ordered
    /// - The curve has duplicate voltage values
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use battery_estimator::{Curve, CurvePoint, Error};
    ///
    /// // Invalid: Only one point
    /// let invalid_curve = Curve::new(&[CurvePoint::new(3.7, 50.0)]);
    /// let result = invalid_curve.voltage_to_soc(3.7);
    /// assert!(matches!(result, Err(Error::InvalidCurve)));
    /// ```
    InvalidCurve,

    /// A numerical error occurred during calculation
    ///
    /// This error occurs when:
    /// - Division by zero in interpolation
    /// - Overflow or underflow in calculations
    /// - Invalid floating-point operations
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use battery_estimator::{Curve, CurvePoint, Error};
    ///
    /// // This could occur if curve points have the same voltage
    /// let problematic_curve = Curve::new(&[
    ///     CurvePoint::new(3.7, 50.0),
    ///     CurvePoint::new(3.7, 60.0), // Duplicate voltage!
    /// ]);
    /// ```
    NumericalError,

    /// The temperature value is invalid
    ///
    /// This error occurs when:
    /// - Temperature is NaN (Not a Number)
    /// - Temperature is infinity
    /// - Temperature is outside reasonable bounds
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use battery_estimator::{BatteryChemistry, SocEstimator, Error};
    ///
    /// let estimator = SocEstimator::new(BatteryChemistry::LiPo);
    ///
    /// // Invalid temperature
    /// let result = estimator.estimate_soc_with_temp(3.7, f32::NAN);
    /// ```
    InvalidTemperature,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::VoltageOutOfRange => write!(f, "Voltage out of valid range"),
            Error::InvalidCurve => write!(f, "Invalid voltage curve"),
            Error::NumericalError => write!(f, "Numerical error in calculation"),
            Error::InvalidTemperature => write!(f, "Invalid temperature"),
        }
    }
}
