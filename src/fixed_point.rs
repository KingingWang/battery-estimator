//! Fixed-point arithmetic support for battery SOC estimation
//!
//! This module provides a high-performance fixed-point alternative to floating-point
//! calculations, suitable for embedded systems without hardware FPU or where
//! deterministic performance is critical.
//!
//! Fixed-point arithmetic offers several advantages over floating-point:
//! - **Faster execution** on systems without FPU (typically 2-10x faster)
//! - **Deterministic performance** - no variable execution time
//! - **Smaller code size** - no floating-point library overhead
//! - **Same precision** for battery estimation use cases
//!
//! # Type Aliases
//!
//! The module uses `FixedI32<U16>` (I16F16) for all calculations:
//! - 16 bits for integer part (range: -32768 to +32767)
//! - 16 bits for fractional part (precision: ~0.000015)
//! - Sufficient for voltage (0-65V), SOC (0-100%), temperature (-40 to +80°C)
//!
//! # Examples
//!
//! ```rust
//! # #[cfg(feature = "fixed-point")]
//! # {
//! use battery_estimator::fixed_point::{FixedSocEstimator, FixedBatteryChemistry, Fixed};
//!
//! // Create estimator for LiPo battery
//! let estimator = FixedSocEstimator::new(FixedBatteryChemistry::LiPo);
//!
//! // Estimate SOC at 3.7V
//! let voltage = Fixed::from_num(3.7);
//! match estimator.estimate_soc(voltage) {
//!     Ok(soc) => println!("Battery SOC: {:.1}%", soc.to_num::<f32>()),
//!     Err(e) => println!("Error: {}", e),
//! }
//! # }
//! ```

#[cfg(feature = "fixed-point")]
use fixed::types::I16F16;

#[cfg(feature = "fixed-point")]
use crate::{BatteryChemistry, Error};

/// Fixed-point number type used throughout this module (I16F16)
///
/// This type provides:
/// - Range: -32768 to +32767
/// - Precision: ~0.000015 (2^-16)
/// - 16 integer bits, 16 fractional bits
#[cfg(feature = "fixed-point")]
pub type Fixed = I16F16;

/// Battery chemistry types for fixed-point operations
///
/// This is a type alias to maintain API consistency with the floating-point version.
#[cfg(feature = "fixed-point")]
pub type FixedBatteryChemistry = BatteryChemistry;

/// A curve point using fixed-point numbers
///
/// Stores voltage and SOC using fixed-point arithmetic for better performance
/// on systems without hardware FPU.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "fixed-point")]
/// # {
/// use battery_estimator::fixed_point::{FixedCurvePoint, Fixed};
///
/// let point = FixedCurvePoint::new(
///     Fixed::from_num(3.7),
///     Fixed::from_num(50.0)
/// );
///
/// assert_eq!(point.voltage().to_num::<f32>(), 3.7);
/// assert_eq!(point.soc().to_num::<f32>(), 50.0);
/// # }
/// ```
#[cfg(feature = "fixed-point")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FixedCurvePoint {
    /// Voltage in volts (fixed-point)
    pub voltage: Fixed,
    /// State of charge in percent (fixed-point)
    pub soc: Fixed,
}

#[cfg(feature = "fixed-point")]
impl FixedCurvePoint {
    /// Creates a new curve point from fixed-point values
    ///
    /// # Arguments
    ///
    /// * `voltage` - Voltage in volts (fixed-point)
    /// * `soc` - State of charge in percent (0.0 to 100.0, fixed-point)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "fixed-point")]
    /// # {
    /// use battery_estimator::fixed_point::{FixedCurvePoint, Fixed};
    ///
    /// let point = FixedCurvePoint::new(
    ///     Fixed::from_num(3.7),
    ///     Fixed::from_num(50.0)
    /// );
    /// # }
    /// ```
    #[inline]
    pub const fn new(voltage: Fixed, soc: Fixed) -> Self {
        Self { voltage, soc }
    }

    /// Creates a new curve point from floating-point values
    ///
    /// Convenience method for creating fixed-point curve points from f32 values.
    ///
    /// # Arguments
    ///
    /// * `voltage` - Voltage in volts (f32)
    /// * `soc` - State of charge in percent (f32)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "fixed-point")]
    /// # {
    /// use battery_estimator::fixed_point::FixedCurvePoint;
    ///
    /// let point = FixedCurvePoint::from_f32(3.7, 50.0);
    /// # }
    /// ```
    #[inline]
    pub fn from_f32(voltage: f32, soc: f32) -> Self {
        Self {
            voltage: Fixed::from_num(voltage),
            soc: Fixed::from_num(soc),
        }
    }

    /// Returns the voltage
    #[inline]
    pub const fn voltage(&self) -> Fixed {
        self.voltage
    }

    /// Returns the state of charge
    #[inline]
    pub const fn soc(&self) -> Fixed {
        self.soc
    }
}

/// Maximum number of points in a fixed-point curve
#[cfg(feature = "fixed-point")]
pub const MAX_FIXED_CURVE_POINTS: usize = 32;

/// A voltage-to-SOC curve using fixed-point arithmetic
///
/// This provides the same functionality as [`Curve`](crate::Curve) but uses
/// fixed-point numbers for better performance on embedded systems.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "fixed-point")]
/// # {
/// use battery_estimator::fixed_point::{FixedCurve, FixedCurvePoint, Fixed};
///
/// let curve = FixedCurve::from_f32_slice(&[
///     (3.0, 0.0),
///     (3.5, 50.0),
///     (4.0, 100.0),
/// ]);
///
/// let voltage = Fixed::from_num(3.5);
/// let soc = curve.voltage_to_soc(voltage).unwrap();
/// assert_eq!(soc.to_num::<f32>(), 50.0);
/// # }
/// ```
#[cfg(feature = "fixed-point")]
#[derive(Debug, Clone, Copy)]
pub struct FixedCurve {
    points: [FixedCurvePoint; MAX_FIXED_CURVE_POINTS],
    len: u8,
    min_voltage: Fixed,
    max_voltage: Fixed,
    min_soc: Fixed,
    max_soc: Fixed,
}

#[cfg(feature = "fixed-point")]
impl FixedCurve {
    /// Creates an empty curve
    pub const fn empty() -> Self {
        Self {
            points: [FixedCurvePoint::new(Fixed::ZERO, Fixed::ZERO); MAX_FIXED_CURVE_POINTS],
            len: 0,
            min_voltage: Fixed::ZERO,
            max_voltage: Fixed::ZERO,
            min_soc: Fixed::ZERO,
            max_soc: Fixed::ZERO,
        }
    }

    /// Creates a new curve from a slice of fixed-point curve points
    ///
    /// # Arguments
    ///
    /// * `points` - Slice of curve points, must be sorted by increasing voltage
    pub fn new(points: &[FixedCurvePoint]) -> Self {
        let mut curve = Self::empty();
        let len = points.len().min(MAX_FIXED_CURVE_POINTS);

        if len == 0 {
            return curve;
        }

        // Copy points and find min/max
        let mut min_v = points[0].voltage;
        let mut max_v = points[0].voltage;
        let mut min_s = points[0].soc;
        let mut max_s = points[0].soc;

        for (i, &point) in points.iter().take(len).enumerate() {
            curve.points[i] = point;

            if point.voltage < min_v {
                min_v = point.voltage;
                min_s = point.soc;
            }
            if point.voltage > max_v {
                max_v = point.voltage;
                max_s = point.soc;
            }
        }

        curve.len = len as u8;
        curve.min_voltage = min_v;
        curve.max_voltage = max_v;
        curve.min_soc = min_s;
        curve.max_soc = max_s;

        curve
    }

    /// Creates a new curve from a slice of (voltage, soc) f32 tuples
    ///
    /// Convenience method for creating fixed-point curves from floating-point data.
    ///
    /// # Arguments
    ///
    /// * `points` - Slice of (voltage, soc) tuples as f32 values
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "fixed-point")]
    /// # {
    /// use battery_estimator::fixed_point::FixedCurve;
    ///
    /// let curve = FixedCurve::from_f32_slice(&[
    ///     (3.0, 0.0),
    ///     (3.5, 50.0),
    ///     (4.0, 100.0),
    /// ]);
    /// # }
    /// ```
    pub fn from_f32_slice(points: &[(f32, f32)]) -> Self {
        let mut fixed_points = [FixedCurvePoint::new(Fixed::ZERO, Fixed::ZERO); MAX_FIXED_CURVE_POINTS];
        let len = points.len().min(MAX_FIXED_CURVE_POINTS);
        
        for (i, &(v, s)) in points.iter().take(len).enumerate() {
            fixed_points[i] = FixedCurvePoint::from_f32(v, s);
        }

        Self::new(&fixed_points[..len])
    }

    /// Converts a voltage to SOC using linear interpolation
    ///
    /// # Arguments
    ///
    /// * `voltage` - Battery voltage (fixed-point)
    ///
    /// # Returns
    ///
    /// * `Ok(soc)` - State of charge percentage (fixed-point)
    /// * `Err(Error::InvalidCurve)` - Curve has fewer than 2 points
    pub fn voltage_to_soc(&self, voltage: Fixed) -> Result<Fixed, Error> {
        if self.len < 2 {
            return Err(Error::InvalidCurve);
        }

        // Boundary checks
        if voltage >= self.max_voltage {
            return Ok(self.max_soc);
        }
        if voltage <= self.min_voltage {
            return Ok(self.min_soc);
        }

        // Binary search for interpolation segment
        let points = &self.points[..self.len as usize];
        let idx = points.partition_point(|p| p.voltage <= voltage);

        if idx > 0 && idx < points.len() {
            let prev = points[idx - 1];
            let curr = points[idx];

            if voltage >= prev.voltage && voltage <= curr.voltage {
                let voltage_range = curr.voltage - prev.voltage;
                let soc_range = curr.soc - prev.soc;
                let ratio = (voltage - prev.voltage) / voltage_range;
                let soc = prev.soc + ratio * soc_range;
                return Ok(soc);
            }
        }

        Err(Error::NumericalError)
    }

    /// Returns the voltage range
    #[inline]
    pub const fn voltage_range(&self) -> (Fixed, Fixed) {
        (self.min_voltage, self.max_voltage)
    }

    /// Returns the number of points in the curve
    #[inline]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns true if the curve is empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Fixed-point SOC estimator configuration
#[cfg(feature = "fixed-point")]
#[derive(Debug, Clone, Copy)]
pub struct FixedEstimatorConfig {
    /// Nominal temperature (°C)
    pub nominal_temperature: Fixed,
    /// Temperature compensation coefficient
    pub temperature_coefficient: Fixed,
    /// Battery age (years)
    pub age_years: Fixed,
    /// Aging factor
    pub aging_factor: Fixed,
    /// Compensation flags
    flags: u8,
}

#[cfg(feature = "fixed-point")]
impl FixedEstimatorConfig {
    /// Default configuration
    pub fn default() -> Self {
        Self {
            nominal_temperature: Fixed::from_num(25.0),
            temperature_coefficient: Fixed::from_num(0.005),
            age_years: Fixed::ZERO,
            aging_factor: Fixed::from_num(0.02),
            flags: 0,
        }
    }

    /// Enable temperature compensation
    #[inline]
    pub fn with_temperature_compensation(mut self) -> Self {
        self.flags |= 0x01;
        self
    }

    /// Enable aging compensation
    #[inline]
    pub fn with_aging_compensation(mut self) -> Self {
        self.flags |= 0x02;
        self
    }

    /// Set nominal temperature
    #[inline]
    pub fn with_nominal_temperature(mut self, temp: Fixed) -> Self {
        self.nominal_temperature = temp;
        self
    }

    /// Set temperature coefficient
    #[inline]
    pub fn with_temperature_coefficient(mut self, coeff: Fixed) -> Self {
        self.temperature_coefficient = coeff;
        self
    }

    /// Set battery age
    #[inline]
    pub fn with_age_years(mut self, years: Fixed) -> Self {
        self.age_years = years;
        self
    }

    /// Set aging factor
    #[inline]
    pub fn with_aging_factor(mut self, factor: Fixed) -> Self {
        self.aging_factor = factor;
        self
    }

    /// Returns true if temperature compensation is enabled
    pub const fn is_temperature_compensation_enabled(&self) -> bool {
        (self.flags & 0x01) != 0
    }

    /// Returns true if aging compensation is enabled
    pub const fn is_aging_compensation_enabled(&self) -> bool {
        (self.flags & 0x02) != 0
    }
}

#[cfg(feature = "fixed-point")]
impl Default for FixedEstimatorConfig {
    fn default() -> Self {
        Self::default()
    }
}

/// Fixed-point SOC estimator
///
/// High-performance battery SOC estimator using fixed-point arithmetic.
/// Suitable for embedded systems without FPU.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "fixed-point")]
/// # {
/// use battery_estimator::fixed_point::{FixedSocEstimator, FixedBatteryChemistry, Fixed};
///
/// let estimator = FixedSocEstimator::new(FixedBatteryChemistry::LiPo);
/// let voltage = Fixed::from_num(3.7);
/// let soc = estimator.estimate_soc(voltage).unwrap();
/// println!("SOC: {:.1}%", soc.to_num::<f32>());
/// # }
/// ```
#[cfg(feature = "fixed-point")]
#[derive(Debug, Clone, Copy)]
pub struct FixedSocEstimator {
    curve: FixedCurve,
    config: FixedEstimatorConfig,
}

#[cfg(feature = "fixed-point")]
impl FixedSocEstimator {
    /// Create a new fixed-point SOC estimator
    ///
    /// # Arguments
    ///
    /// * `chemistry` - Battery chemistry type
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "fixed-point")]
    /// # {
    /// use battery_estimator::fixed_point::{FixedSocEstimator, FixedBatteryChemistry};
    ///
    /// let estimator = FixedSocEstimator::new(FixedBatteryChemistry::LiPo);
    /// # }
    /// ```
    pub fn new(chemistry: BatteryChemistry) -> Self {
        let curve = match chemistry {
            BatteryChemistry::LiPo => Self::create_lipo_curve(),
            BatteryChemistry::LiFePO4 => Self::create_lifepo4_curve(),
            BatteryChemistry::LiIon => Self::create_liion_curve(),
            BatteryChemistry::Lipo410Full340Cutoff => Self::create_lipo410_curve(),
        };

        Self {
            curve,
            config: FixedEstimatorConfig::default(),
        }
    }

    /// Create estimator with custom curve
    pub fn with_custom_curve(curve: FixedCurve) -> Self {
        Self {
            curve,
            config: FixedEstimatorConfig::default(),
        }
    }

    /// Create estimator with configuration
    pub fn with_config(chemistry: BatteryChemistry, config: FixedEstimatorConfig) -> Self {
        let curve = match chemistry {
            BatteryChemistry::LiPo => Self::create_lipo_curve(),
            BatteryChemistry::LiFePO4 => Self::create_lifepo4_curve(),
            BatteryChemistry::LiIon => Self::create_liion_curve(),
            BatteryChemistry::Lipo410Full340Cutoff => Self::create_lipo410_curve(),
        };

        Self { curve, config }
    }

    // Helper methods to create battery curves
    fn create_lipo_curve() -> FixedCurve {
        FixedCurve::from_f32_slice(&[
            (3.20, 0.0),
            (3.30, 5.0),
            (3.40, 10.0),
            (3.50, 20.0),
            (3.60, 30.0),
            (3.70, 50.0),
            (3.80, 70.0),
            (3.90, 85.0),
            (4.00, 95.0),
            (4.20, 100.0),
        ])
    }

    fn create_lifepo4_curve() -> FixedCurve {
        FixedCurve::from_f32_slice(&[
            (2.50, 0.0),
            (2.80, 15.0),
            (3.00, 35.0),
            (3.10, 45.0),
            (3.20, 55.0),
            (3.30, 65.0),
            (3.40, 75.0),
            (3.50, 85.0),
            (3.60, 95.0),
            (3.65, 100.0),
        ])
    }

    fn create_liion_curve() -> FixedCurve {
        FixedCurve::from_f32_slice(&[
            (2.50, 0.0),
            (3.00, 30.0),
            (3.30, 50.0),
            (3.50, 65.0),
            (3.60, 70.0),
            (3.70, 75.0),
            (3.80, 80.0),
            (3.90, 85.0),
            (4.00, 90.0),
            (4.10, 95.0),
            (4.20, 100.0),
        ])
    }

    fn create_lipo410_curve() -> FixedCurve {
        FixedCurve::from_f32_slice(&[
            (3.40, 0.0),
            (3.48, 5.0),
            (3.53, 10.0),
            (3.62, 20.0),
            (3.68, 30.0),
            (3.73, 40.0),
            (3.77, 50.0),
            (3.81, 60.0),
            (3.85, 70.0),
            (3.90, 80.0),
            (3.97, 90.0),
            (4.03, 95.0),
            (4.10, 100.0),
        ])
    }

    /// Estimate SOC from voltage
    ///
    /// # Arguments
    ///
    /// * `voltage` - Battery voltage (fixed-point)
    ///
    /// # Returns
    ///
    /// State of charge percentage (fixed-point)
    pub fn estimate_soc(&self, voltage: Fixed) -> Result<Fixed, Error> {
        self.curve.voltage_to_soc(voltage)
    }

    /// Estimate SOC with compensation
    ///
    /// # Arguments
    ///
    /// * `voltage` - Battery voltage (fixed-point)
    /// * `temperature` - Current temperature in Celsius (fixed-point)
    ///
    /// # Returns
    ///
    /// Compensated state of charge percentage (fixed-point)
    pub fn estimate_soc_compensated(
        &self,
        voltage: Fixed,
        temperature: Fixed,
    ) -> Result<Fixed, Error> {
        let mut soc = self.curve.voltage_to_soc(voltage)?;

        // Apply temperature compensation
        if self.config.is_temperature_compensation_enabled() {
            soc = compensate_temperature_fixed(
                soc,
                temperature,
                self.config.nominal_temperature,
                self.config.temperature_coefficient,
            );
        }

        // Apply aging compensation
        if self.config.is_aging_compensation_enabled() {
            soc = compensate_aging_fixed(soc, self.config.age_years, self.config.aging_factor);
        }

        // Clamp to valid range
        let zero = Fixed::ZERO;
        let hundred = Fixed::from_num(100);
        Ok(soc.clamp(zero, hundred))
    }

    /// Get voltage range
    #[inline]
    pub const fn voltage_range(&self) -> (Fixed, Fixed) {
        self.curve.voltage_range()
    }

    /// Get current configuration
    #[inline]
    pub const fn config(&self) -> &FixedEstimatorConfig {
        &self.config
    }

    /// Update configuration
    #[inline]
    pub fn update_config(&mut self, config: FixedEstimatorConfig) {
        self.config = config;
    }
}

/// Apply temperature compensation using fixed-point arithmetic
///
/// # Arguments
///
/// * `soc` - Base SOC percentage (fixed-point)
/// * `temperature` - Current temperature (fixed-point)
/// * `nominal_temp` - Nominal temperature (fixed-point)
/// * `coefficient` - Temperature coefficient (fixed-point)
#[cfg(feature = "fixed-point")]
pub fn compensate_temperature_fixed(
    soc: Fixed,
    temperature: Fixed,
    nominal_temp: Fixed,
    coefficient: Fixed,
) -> Fixed {
    let delta_temp = temperature - nominal_temp;

    let capacity_change = if delta_temp < Fixed::ZERO {
        // Cold: reduce capacity
        delta_temp * coefficient
    } else {
        // Warm: slight capacity increase (capped at 5%)
        let half = Fixed::from_num(0.5);
        let max_increase = Fixed::from_num(0.05);
        (delta_temp * coefficient * half).min(max_increase)
    };

    // Bound the compensation
    let min_change = Fixed::from_num(-0.30);
    let max_change = Fixed::from_num(0.05);
    let bounded_change = capacity_change.clamp(min_change, max_change);

    let one = Fixed::from_num(1.0);
    soc * (one + bounded_change)
}

/// Apply aging compensation using fixed-point arithmetic
///
/// # Arguments
///
/// * `soc` - Base SOC percentage (fixed-point)
/// * `age_years` - Battery age in years (fixed-point)
/// * `aging_factor` - Aging factor (fixed-point)
#[cfg(feature = "fixed-point")]
pub fn compensate_aging_fixed(soc: Fixed, age_years: Fixed, aging_factor: Fixed) -> Fixed {
    if age_years < Fixed::ZERO || aging_factor < Fixed::ZERO {
        return soc;
    }

    let age_compensation = age_years * aging_factor;
    let max_comp = Fixed::from_num(0.5);
    let bounded_comp = age_compensation.clamp(Fixed::ZERO, max_comp);

    let one = Fixed::from_num(1.0);
    soc * (one - bounded_comp)
}

#[cfg(all(test, feature = "fixed-point"))]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_curve_point() {
        let point = FixedCurvePoint::from_f32(3.7, 50.0);
        assert!((point.voltage().to_num::<f32>() - 3.7).abs() < 0.001);
        assert!((point.soc().to_num::<f32>() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_fixed_curve_basic() {
        let curve = FixedCurve::from_f32_slice(&[(3.0, 0.0), (4.0, 100.0)]);

        let v3 = Fixed::from_num(3.0);
        let v4 = Fixed::from_num(4.0);
        let v35 = Fixed::from_num(3.5);

        assert!((curve.voltage_to_soc(v3).unwrap().to_num::<f32>() - 0.0).abs() < 0.1);
        assert!((curve.voltage_to_soc(v4).unwrap().to_num::<f32>() - 100.0).abs() < 0.1);
        assert!((curve.voltage_to_soc(v35).unwrap().to_num::<f32>() - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_fixed_estimator_basic() {
        let estimator = FixedSocEstimator::new(BatteryChemistry::LiPo);
        let voltage = Fixed::from_num(3.7);

        let soc = estimator.estimate_soc(voltage).unwrap();
        let soc_f32 = soc.to_num::<f32>();

        assert!(
            (45.0..=55.0).contains(&soc_f32),
            "Expected ~50%, got {:.1}%",
            soc_f32
        );
    }

    #[test]
    fn test_fixed_temperature_compensation() {
        let estimator = FixedSocEstimator::new(BatteryChemistry::LiPo);
        let voltage = Fixed::from_num(3.7);
        let temp_cold = Fixed::from_num(0.0);
        let temp_nominal = Fixed::from_num(25.0);

        let base_soc = estimator.estimate_soc(voltage).unwrap();
        let comp_soc = compensate_temperature_fixed(
            base_soc,
            temp_cold,
            temp_nominal,
            Fixed::from_num(0.005),
        );

        assert!(comp_soc < base_soc, "Cold temperature should reduce SOC");
    }

    #[test]
    fn test_fixed_aging_compensation() {
        let soc = Fixed::from_num(50.0);
        let age = Fixed::from_num(5.0);
        let factor = Fixed::from_num(0.02);

        let aged_soc = compensate_aging_fixed(soc, age, factor);

        assert!(aged_soc < soc, "Aging should reduce SOC");
    }

    #[test]
    fn test_fixed_estimator_with_compensation() {
        let config = FixedEstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(Fixed::from_num(25.0))
            .with_temperature_coefficient(Fixed::from_num(0.005));

        let estimator = FixedSocEstimator::with_config(BatteryChemistry::LiPo, config);

        let voltage = Fixed::from_num(3.7);
        let temp = Fixed::from_num(0.0); // Cold temperature

        let soc = estimator.estimate_soc_compensated(voltage, temp).unwrap();
        let base_soc = estimator.estimate_soc(voltage).unwrap();

        assert!(soc < base_soc, "Compensated SOC should be lower in cold");
    }
}
