//! Error types for battery SOC estimation
//!
//! This module defines the error types that can occur during battery
//! state-of-charge estimation operations.

use thiserror::Error;

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
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    #[error("Voltage out of valid range")]
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
    #[error("Invalid voltage curve")]
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
    #[error("Numerical error in calculation")]
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
    #[error("Invalid temperature")]
    InvalidTemperature,
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Write;

    #[test]
    fn test_error_display() {
        // In no-std, Display is available via core::fmt
        // We can test that the Display implementation compiles and works
        let mut buffer = [0u8; 64];

        // Test VoltageOutOfRange
        let mut writer = BufferWriter::new(&mut buffer);
        write!(writer, "{}", Error::VoltageOutOfRange).unwrap();
        assert_eq!(writer.as_str(), "Voltage out of valid range");

        // Test InvalidCurve
        let mut writer = BufferWriter::new(&mut buffer);
        write!(writer, "{}", Error::InvalidCurve).unwrap();
        assert_eq!(writer.as_str(), "Invalid voltage curve");

        // Test NumericalError
        let mut writer = BufferWriter::new(&mut buffer);
        write!(writer, "{}", Error::NumericalError).unwrap();
        assert_eq!(writer.as_str(), "Numerical error in calculation");

        // Test InvalidTemperature
        let mut writer = BufferWriter::new(&mut buffer);
        write!(writer, "{}", Error::InvalidTemperature).unwrap();
        assert_eq!(writer.as_str(), "Invalid temperature");
    }

    #[test]
    fn test_error_equality() {
        assert_eq!(Error::VoltageOutOfRange, Error::VoltageOutOfRange);
        assert_eq!(Error::InvalidCurve, Error::InvalidCurve);
        assert_eq!(Error::NumericalError, Error::NumericalError);
        assert_eq!(Error::InvalidTemperature, Error::InvalidTemperature);

        assert_ne!(Error::VoltageOutOfRange, Error::InvalidCurve);
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
        let mut buffer = [0u8; 64];
        let mut writer = BufferWriter::new(&mut buffer);
        write!(writer, "{:?}", error).unwrap();
        assert!(writer.as_str().contains("NumericalError"));
    }

    #[test]
    fn test_error_all_variants() {
        // Test that all error variants can be created
        let errors = [
            Error::VoltageOutOfRange,
            Error::InvalidCurve,
            Error::NumericalError,
            Error::InvalidTemperature,
        ];
        assert_eq!(errors.len(), 4);
    }

    #[test]
    fn test_error_variants_distinct() {
        let error1 = Error::VoltageOutOfRange;
        let error2 = Error::InvalidCurve;
        let error3 = Error::NumericalError;
        let error4 = Error::InvalidTemperature;

        // Verify all variants are distinct
        assert_ne!(error1, error2);
        assert_ne!(error2, error3);
        assert_ne!(error3, error4);
        assert_ne!(error1, error3);
        assert_ne!(error1, error4);
        assert_ne!(error2, error4);
    }

    // Helper struct for testing Display in no-std
    struct BufferWriter<'a> {
        buffer: &'a mut [u8],
        pos: usize,
    }

    impl<'a> BufferWriter<'a> {
        fn new(buffer: &'a mut [u8]) -> Self {
            BufferWriter { buffer, pos: 0 }
        }

        fn as_str(&self) -> &str {
            core::str::from_utf8(&self.buffer[..self.pos]).unwrap()
        }
    }

    impl<'a> core::fmt::Write for BufferWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            if self.pos + bytes.len() > self.buffer.len() {
                return Err(core::fmt::Error);
            }
            self.buffer[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
            self.pos += bytes.len();
            Ok(())
        }
    }
}
