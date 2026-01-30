# Battery SOC Estimator

[![Crates.io](https://img.shields.io/crates/v/battery-estimator)](https://crates.io/crates/battery-estimator)
[![Documentation](https://docs.rs/battery-estimator/badge.svg)](https://docs.rs/battery-estimator)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A lightweight, `no_std` compatible Rust library for estimating battery State of Charge (SOC) from voltage measurements. Designed specifically for embedded systems with zero dependencies and no heap allocations.

## Features

- ✅ **Minimal dependencies** - Only depends on `thiserror` for error handling (no runtime overhead)
- ✅ **`no_std` compatible** - Perfect for embedded systems and microcontrollers
- ✅ **No heap allocations** - Uses only stack memory and fixed-size arrays
- ✅ **Multiple battery chemistries** - Built-in support for LiPo, LiFePO4, Li-Ion, and conservative curves
- ✅ **Temperature compensation** - Correct SOC readings based on temperature effects
- ✅ **Aging compensation** - Adjust for battery capacity degradation over time
- ✅ **Custom voltage curves** - Define your own voltage-SOC relationships
- ✅ **Input validation** - Safe handling of invalid inputs (NaN, Infinity, negative values)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
battery-estimator = "0.1"
```

## Quick Start

### Basic Usage

```rust
use battery_estimator::{BatteryChemistry, SocEstimator};

fn main() {
    // Create estimator for a standard LiPo battery
    let estimator = SocEstimator::new(BatteryChemistry::LiPo);
    
    // Estimate SOC at 3.7V
    match estimator.estimate_soc(3.7) {
        Ok(soc) => println!("Battery SOC: {:.1}%", soc),
        Err(e) => println!("Error estimating SOC: {}", e),
    }
}
```

### Temperature Compensation

```rust
use battery_estimator::{BatteryChemistry, SocEstimator};

fn main() {
    // Create estimator with temperature compensation enabled
    let estimator = SocEstimator::with_temperature_compensation(
        BatteryChemistry::LiPo,
        25.0,  // Nominal temperature (°C)
        0.005  // Temperature coefficient (0.5% capacity loss per °C below nominal)
    );
    
    // Estimate SOC at 3.7V with current temperature of 20°C
    match estimator.estimate_soc_compensated(3.7, 20.0) {
        Ok(soc) => println!("Temperature-compensated SOC: {:.1}%", soc),
        Err(e) => println!("Error: {}", e),
    }
}
```

**Note**: `estimate_soc_compensated` only applies compensation when the estimator is configured with compensation enabled. Use `with_temperature_compensation()` or `with_all_compensation()` to enable it.

### Custom Voltage Curve

```rust
use battery_estimator::{SocEstimator, Curve, CurvePoint};

fn main() {
    // Define a custom voltage-SOC curve
    const CUSTOM_CURVE: Curve = Curve::new(&[
        CurvePoint::new(3.0, 0.0),   // 3.0V = 0%
        CurvePoint::new(3.5, 50.0),  // 3.5V = 50%
        CurvePoint::new(4.0, 100.0), // 4.0V = 100%
    ]);
    
    // Create estimator with custom curve
    let estimator = SocEstimator::with_custom_curve(&CUSTOM_CURVE);
    
    // Use the estimator
    match estimator.estimate_soc(3.75) {
        Ok(soc) => println!("Custom curve SOC: {:.1}%", soc),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Supported Battery Types

| Battery Type | Full Charge | Cutoff Voltage | Description |
|-------------|-------------|----------------|-------------|
| `LiPo` | 4.2V | 3.2V | Standard Lithium Polymer battery |
| `LiFePO4` | 3.65V | 3.0V | Lithium Iron Phosphate battery (longer cycle life) |
| `LiIon` | 4.2V | 3.3V | Standard Lithium Ion battery |
| `Lipo410Full340Cutoff` | 4.1V | 3.4V | Conservative LiPo curve (extended battery life) |

### Conservative Battery Curve

The `Lipo410Full340Cutoff` variant uses conservative charge/discharge thresholds:
- **Lower full charge voltage** (4.1V instead of 4.2V) - Reduces stress on battery
- **Higher cutoff voltage** (3.4V instead of 3.2V) - Prevents deep discharge
- **Result**: Extended battery cycle life at the cost of reduced usable capacity

## Advanced Usage

### Aging Compensation

```rust
use battery_estimator::{BatteryChemistry, SocEstimator};

fn main() {
    // Create estimator with aging compensation for a 2-year-old battery
    let estimator = SocEstimator::with_aging_compensation(
        BatteryChemistry::LiPo,
        2.0,   // Battery age in years
        0.02   // Aging factor (2% capacity loss per year)
    );
    
    match estimator.estimate_soc_compensated(3.7, 25.0) {
        Ok(soc) => println!("Age-compensated SOC: {:.1}%", soc),
        Err(e) => println!("Error: {}", e),
    }
}
```

**Note**: `estimate_soc_compensated` only applies compensation when the estimator is configured with compensation enabled. Use `with_aging_compensation()` or `with_all_compensation()` to enable it.

### Combined Compensation

```rust
use battery_estimator::{BatteryChemistry, SocEstimator};

fn main() {
    // Create estimator with both temperature and aging compensation
    let estimator = SocEstimator::with_all_compensation(
        BatteryChemistry::LiPo,
        25.0,  // Nominal temperature
        0.005, // Temperature coefficient (0.5% capacity loss per °C below nominal)
        2.0,   // Battery age in years
        0.02   // Aging factor (2% capacity loss per year)
    );
    
    match estimator.estimate_soc_compensated(3.7, 20.0) {
        Ok(soc) => println!("Fully compensated SOC: {:.1}%", soc),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Memory Footprint

The library is designed for minimal memory usage:

- **No heap allocations**: All data is stack-allocated
- **Optimized for embedded**: Uses `u8` and `u16` where possible instead of larger integers

## Performance

- **Fast estimation**: O(n) where n is the number of curve points (typically 8-12)
- **Deterministic execution time**: No dynamic memory allocation
- **Linear search**: Efficient for typical battery curves with limited points
- **Const-friendly**: Curve creation and validation use `const fn` for compile-time safety

## API Documentation

For detailed API documentation, visit [docs.rs](https://docs.rs/battery-estimator).

## Examples

See the `examples/` directory for complete examples:

- `basic.rs` - Comprehensive testing of all battery types
- `custom_curve.rs` - Using custom voltage curves
- `precise_test.rs` - Precise voltage testing with 0.01V resolution
- `temperature_compensation_test.rs` - Temperature compensation demonstration

Run examples with:

```bash
cargo run --example basic
cargo run --example custom_curve
cargo run --example precise_test
cargo run --example temperature_compensation_test
```

## Testing

Run the test suite:

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Designed for embedded systems and microcontrollers
- Optimized for minimal memory footprint and fast execution
- Tested on various battery chemistries and voltage curves
