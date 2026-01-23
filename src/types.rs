//! Type definitions for battery SOC estimation
//!
//! This module contains the core data types used throughout the library:
//!
//! - [`BatteryChemistry`] - Enumeration of supported battery types
//! - [`CurvePoint`] - Individual voltage-SOC data point for curves

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
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// * `voltage` - Voltage in volts (e.g., 3.7)
    /// * `soc` - State of charge in percent (e.g., 50.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use battery_estimator::CurvePoint;
    ///
    /// let point = CurvePoint::new(3.7, 50.0);
    /// ```
    #[inline]
    pub const fn new(voltage: f32, soc: f32) -> Self {
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
