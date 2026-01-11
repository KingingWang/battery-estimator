//! # Battery SOC Estimator
//!
//! A zero-dependency, `no_std` compatible library for estimating battery
//! state-of-charge (SOC) from voltage measurements.
//!
//! ## Features
//! - Zero dependencies
//! - `no_std` compatible
//! - No heap allocations
//! - Multiple battery chemistries
//! - Custom voltage curves
//! - Temperature compensation
//!
//! ## Example
//! ```
//! use battery_estimator::{BatteryChemistry, SocEstimator};
//!
//! let estimator = SocEstimator::new(BatteryChemistry::LiPo);
//! let soc = estimator.estimate_soc(3.7).unwrap();
//! println!("SOC: {:.1}%", soc); // ~30%
//! ```

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
pub use curve::{Curve, default_curves};
pub use error::Error;
pub use estimator::SocEstimator;
pub use types::{BatteryChemistry, CurvePoint};

/// 重新导出核心类型，方便使用
pub mod prelude {
    pub use crate::{BatteryChemistry, CurvePoint, Error, SocEstimator};
}
