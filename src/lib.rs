//! # Battery SOC (State of Charge) Estimator
//!
//! A lightweight, `no_std` compatible Rust library for estimating
//! battery state-of-charge (SOC) from voltage measurements. Designed specifically
//! for embedded systems and microcontrollers using **fixed-point arithmetic**
//! for maximum efficiency without FPU.
//!
//! ## Features
//!
//! - **Fixed-point arithmetic** - Uses `fixed` crate for efficient integer-based calculations
//! - **No floating-point unit required** - Perfect for microcontrollers without FPU
//! - **`no_std` compatible** - Works in embedded environments
//! - **No heap allocations** - Uses only stack memory and fixed-size arrays
//! - **Multiple battery chemistries** - Built-in support for LiPo, LiFePO4, Li-Ion
//! - **Temperature compensation** - Correct SOC readings based on temperature
//! - **Aging compensation** - Adjust for battery capacity degradation over time
//! - **Custom voltage curves** - Define your own voltage-SOC relationships
//! - **Conservative battery curves** - Extended battery life with conservative thresholds
//!
//! ## Quick Start
//!
//! ```rust
//! use battery_estimator::{BatteryChemistry, SocEstimator};
//!
//! // Create estimator for a standard LiPo battery
//! let estimator = SocEstimator::new(BatteryChemistry::LiPo);
//!
//! // Estimate SOC at 3.7V (nominal voltage)
//! match estimator.estimate_soc(3.7) {
//!     Ok(soc) => println!("Battery SOC: {:.1}%", soc),
//!     Err(e) => println!("Error estimating SOC: {}", e),
//! }
//! ```
//!
//! ## Fixed-Point API
//!
//! For embedded systems without FPU, use the fixed-point API for maximum efficiency:
//!
//! ```rust
//! use battery_estimator::{BatteryChemistry, SocEstimator, Fixed};
//! use fixed::types::I16F16;
//!
//! let estimator = SocEstimator::new(BatteryChemistry::LiPo);
//!
//! // Use fixed-point voltage for calculations
//! let voltage = Fixed::from_num(3.7);
//! let soc = estimator.estimate_soc_fixed(voltage).unwrap();
//!
//! // Convert back to float if needed
//! println!("Battery SOC: {:.1}%", soc.to_num::<f32>());
//! ```
//!
//! ## Battery Types
//!
//! The library supports multiple battery chemistries with built-in voltage curves:
//!
//! | Type | Full Charge | Cutoff | Description |
//! |------|-------------|--------|-------------|
//! | `LiPo` | 4.2V | 3.2V | Standard Lithium Polymer |
//! | `LiFePO4` | 3.65V | 3.0V | Lithium Iron Phosphate (long cycle life) |
//! | `LiIon` | 4.2V | 3.3V | Standard Lithium Ion |
//! | `Lipo410Full340Cutoff` | 4.1V | 3.4V | Conservative LiPo (extended life) |
//!
//! ## Temperature Compensation
//!
//! ```rust
//! use battery_estimator::{BatteryChemistry, SocEstimator, Fixed};
//!
//! // Create estimator with temperature compensation
//! let estimator = SocEstimator::with_temperature_compensation(
//!     BatteryChemistry::LiPo,
//!     Fixed::from_num(25.0), // Nominal temperature (°C)
//!     Fixed::from_num(0.0005) // Temperature coefficient
//! );
//!
//! // Estimate SOC with current temperature
//! match estimator.estimate_soc_compensated(3.7, 20.0) {
//!     Ok(soc) => println!("Temperature-compensated SOC: {:.1}%", soc),
//!     Err(e) => println!("Error: {}", e),
//! }
//! ```
//!
//! ## Custom Voltage Curves
//!
//! ```rust
//! use battery_estimator::{SocEstimator, Curve, CurvePoint};
//!
//! // Define a custom voltage-SOC curve
//! const CUSTOM_CURVE: Curve = Curve::new(&[
//!     CurvePoint::new(3.0, 0.0),   // 3.0V = 0%
//!     CurvePoint::new(3.5, 50.0),  // 3.5V = 50%
//!     CurvePoint::new(4.0, 100.0), // 4.0V = 100%
//! ]);
//!
//! // Create estimator with custom curve
//! let estimator = SocEstimator::with_custom_curve(&CUSTOM_CURVE);
//! ```
//!
//! ## Module Structure
//!
//! - [`SocEstimator`] - Main estimator struct for SOC calculations
//! - [`EstimatorConfig`] - Configuration for SOC estimator (compensation settings)
//! - [`BatteryChemistry`] - Supported battery types
//! - [`Curve`] - Voltage-SOC curve representation
//! - [`CurvePoint`] - Individual voltage-SOC data point
//! - [`Fixed`] - Fixed-point type alias (I16F16)
//! - [`Error`] - Error types for estimation failures
//! - [`compensate_temperature`] - Temperature compensation function
//! - [`compensate_aging`] - Aging compensation function

#![no_std]
#![deny(missing_docs, unsafe_code)]

mod compensation;
mod curve;
mod error;
mod estimator;
mod types;

pub use compensation::{
    compensate_aging, compensate_aging_fixed, compensate_temperature, compensate_temperature_fixed,
    default_temperature_compensation, default_temperature_compensation_fixed,
};
pub use curve::{Curve, MAX_CURVE_POINTS};
pub use error::Error;
pub use estimator::{EstimatorConfig, SocEstimator};
pub use types::{BatteryChemistry, CurvePoint, Fixed};

// Re-export the fixed type for convenience
pub use fixed::types::I16F16;

/// Prelude module for convenient imports
///
/// This module re-exports the most commonly used types and functions:
///
/// ```
/// use battery_estimator::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        compensate_aging, compensate_aging_fixed, compensate_temperature,
        compensate_temperature_fixed, default_temperature_compensation,
        default_temperature_compensation_fixed, BatteryChemistry, Curve, CurvePoint, Error,
        EstimatorConfig, Fixed, SocEstimator,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::string::ToString;

    #[test]
    fn test_prelude_exports() {
        // Test that prelude exports are accessible
        use crate::prelude::*;

        let _estimator = SocEstimator::new(BatteryChemistry::LiPo);
        let _config = EstimatorConfig::default();
        let _curve = Curve::new(&[CurvePoint::new(3.0, 0.0), CurvePoint::new(4.0, 100.0)]);
        let _point = CurvePoint::new(3.7, 50.0);

        // Test that compensation functions are available
        let _temp_comp = compensate_temperature(50.0, 25.0, 25.0, 0.005);
        let _aging_comp = compensate_aging(50.0, 1.0, 0.02);

        // Test that fixed-point functions are available
        let _temp_comp_fixed = compensate_temperature_fixed(
            Fixed::from_num(50.0),
            Fixed::from_num(25.0),
            Fixed::from_num(25.0),
            Fixed::from_num(0.005),
        );
        let _aging_comp_fixed = compensate_aging_fixed(
            Fixed::from_num(50.0),
            Fixed::from_num(1.0),
            Fixed::from_num(0.02),
        );
    }

    #[test]
    fn test_basic_usage() {
        let estimator = SocEstimator::new(BatteryChemistry::LiPo);

        // Test basic SOC estimation
        let soc = estimator.estimate_soc(3.7).unwrap();
        assert!(soc > 40.0 && soc < 60.0);

        // Test fixed-point SOC estimation
        let soc_fixed = estimator.estimate_soc_fixed(Fixed::from_num(3.7)).unwrap();
        assert!(soc_fixed > Fixed::from_num(40.0) && soc_fixed < Fixed::from_num(60.0));
    }

    #[test]
    fn test_temperature_compensation_usage() {
        let estimator = SocEstimator::with_temperature_compensation(
            BatteryChemistry::LiPo,
            Fixed::from_num(25.0),
            Fixed::from_num(0.005),
        );

        // Test temperature-compensated SOC
        let soc_normal = estimator.estimate_soc_compensated(3.7, 25.0).unwrap();
        let soc_cold = estimator.estimate_soc_compensated(3.7, 0.0).unwrap();

        // Cold temperature should reduce SOC
        assert!(soc_cold < soc_normal);
    }

    #[test]
    fn test_aging_compensation_usage() {
        let estimator = SocEstimator::with_aging_compensation(
            BatteryChemistry::LiPo,
            Fixed::from_num(2.0),
            Fixed::from_num(0.02),
        );

        // Test aging-compensated SOC
        let soc = estimator.estimate_soc_compensated(3.7, 25.0).unwrap();

        // Aged battery should have lower SOC
        assert!(soc > 0.0 && soc < 100.0);
    }

    #[test]
    fn test_all_battery_chemistries() {
        // Test all battery chemistry types
        let chemistries = [
            BatteryChemistry::LiPo,
            BatteryChemistry::LiFePO4,
            BatteryChemistry::LiIon,
            BatteryChemistry::Lipo410Full340Cutoff,
        ];

        for chemistry in chemistries {
            let estimator = SocEstimator::new(chemistry);
            let (min, max) = estimator.voltage_range();

            // Test SOC estimation at midpoint
            let mid_voltage = (min + max) / 2.0;
            let soc = estimator.estimate_soc(mid_voltage).unwrap();
            assert!((0.0..=100.0).contains(&soc));
        }
    }

    #[test]
    fn test_fixed_type_export() {
        // Test that Fixed type is properly exported
        let value: Fixed = Fixed::from_num(3.7);
        // Fixed-point has limited precision, use tolerance
        assert!((value.to_num::<f32>() - 3.7).abs() < 0.001);
    }

    #[test]
    fn test_curve_export() {
        // Test that Curve is properly exported
        let curve = Curve::new(&[CurvePoint::new(3.0, 0.0), CurvePoint::new(4.0, 100.0)]);

        assert_eq!(curve.voltage_to_soc(3.5).unwrap(), 50.0);
    }

    #[test]
    fn test_error_export() {
        // Test that Error is properly exported
        let error = Error::InvalidCurve;
        assert_eq!(error.to_string(), "Invalid voltage curve");
    }

    #[test]
    fn test_config_export() {
        // Test that EstimatorConfig is properly exported
        let config = EstimatorConfig::default()
            .with_temperature_compensation()
            .with_nominal_temperature(Fixed::from_num(30.0));

        assert!(config.is_temperature_compensation_enabled());
        assert_eq!(config.nominal_temperature, Fixed::from_num(30.0));
    }

    #[test]
    fn test_custom_curve() {
        const CUSTOM: Curve = Curve::new(&[
            CurvePoint::new(3.0, 0.0),
            CurvePoint::new(3.5, 50.0),
            CurvePoint::new(4.0, 100.0),
        ]);

        let estimator = SocEstimator::with_custom_curve(&CUSTOM);

        assert_eq!(estimator.estimate_soc(3.0).unwrap(), 0.0);
        assert_eq!(estimator.estimate_soc(3.5).unwrap(), 50.0);
        assert_eq!(estimator.estimate_soc(4.0).unwrap(), 100.0);
    }
}
