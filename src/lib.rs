//! # Battery SOC (State of Charge) Estimator
//!
//! A lightweight, zero-dependency, `no_std` compatible Rust library for estimating
//! battery state-of-charge (SOC) from voltage measurements. Designed specifically
//! for embedded systems and microcontrollers.
//!
//! ## Features
//!
//! - **Zero dependencies** - No external crates required
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
//! use battery_estimator::{BatteryChemistry, SocEstimator};
//!
//! // Create estimator with temperature compensation
//! let estimator = SocEstimator::with_temperature_compensation(
//!     BatteryChemistry::LiPo,
//!     25.0,  // Nominal temperature (°C)
//!     0.0005 // Temperature coefficient
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
//! ## Module Structure
//!
//! - [`SocEstimator`] - Main estimator struct for SOC calculations
//! - [`BatteryChemistry`] - Supported battery types
//! - [`Curve`] - Voltage-SOC curve representation
//! - [`CurvePoint`] - Individual voltage-SOC data point
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
    compensate_aging, compensate_temperature, default_temperature_compensation,
};
pub use curve::{default_curves, Curve};
pub use error::Error;
pub use estimator::SocEstimator;
pub use types::{BatteryChemistry, CurvePoint};

/// Prelude module for convenient imports
///
/// This module re-exports the most commonly used types and traits,
/// allowing you to import them with a single `use` statement:
///
/// ```rust
/// use battery_estimator::prelude::*;
///
/// let estimator = SocEstimator::new(BatteryChemistry::LiPo);
/// ```
pub mod prelude {
    pub use crate::{BatteryChemistry, CurvePoint, Error, SocEstimator};
}
