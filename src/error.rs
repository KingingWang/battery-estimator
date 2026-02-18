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
///     Err(Error::InvalidTemperature) => eprintln!("Invalid temperature"),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
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
            Error::InvalidCurve => write!(f, "Invalid voltage curve"),
            Error::NumericalError => write!(f, "Numerical error in calculation"),
            Error::InvalidTemperature => write!(f, "Invalid temperature"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::string::ToString;

    #[test]
    fn test_error_display() {
        assert_eq!(Error::InvalidCurve.to_string(), "Invalid voltage curve");
        assert_eq!(
            Error::NumericalError.to_string(),
            "Numerical error in calculation"
        );
        assert_eq!(Error::InvalidTemperature.to_string(), "Invalid temperature");
    }

    #[test]
    fn test_error_equality() {
        assert_eq!(Error::InvalidCurve, Error::InvalidCurve);
        assert_eq!(Error::NumericalError, Error::NumericalError);
        assert_eq!(Error::InvalidTemperature, Error::InvalidTemperature);
        assert_ne!(Error::InvalidCurve, Error::NumericalError);
    }

    #[test]
    fn test_error_copy() {
        let error1 = Error::InvalidCurve;
        let error2 = error1;

        assert_eq!(error1, error2);
    }

    #[test]
    fn test_error_debug() {
        let error = Error::NumericalError;
        let debug_str = alloc::format!("{:?}", error);
        assert!(debug_str.contains("NumericalError"));
    }

    #[test]
    fn test_error_all_variants() {
        // Test that all error variants can be created
        let errors = [
            Error::InvalidCurve,
            Error::NumericalError,
            Error::InvalidTemperature,
        ];

        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn test_error_variants_distinct() {
        let error1 = Error::InvalidCurve;
        let error2 = Error::NumericalError;
        let error3 = Error::InvalidTemperature;

        // Verify all variants are distinct
        assert_ne!(error1, error2);
        assert_ne!(error2, error3);
        assert_ne!(error1, error3);
    }
}
