# Battery SOC Estimator

A no_std, zero-dependency library for estimating battery state of charge based on voltage. It includes temperature compensation and aging compensation features.

## Features

- ✅ **Zero dependencies** - No external crates required
- ✅ **`no_std` compatible** - Perfect for embedded systems
- ✅ **No heap allocations** - Fixed-size arrays only
- ✅ **Multiple battery types** - LiPo, LiFePO4, Li-Ion
- ✅ **Temperature compensation** - Basic temperature correction
- ✅ **Custom curves** - Define your own voltage-SOC points

## Installation

```toml
[dependencies]
battery-estimator = "0.1"
```

## Quick Start

```rust
use battery_estimator::{BatteryChemistry, SocEstimator};
fn main() {
    // Create estimator for LiPo battery
    let estimator = SocEstimator::new(BatteryChemistry::LiPo);
    
    // Estimate SOC at 3.7V
    match estimator.estimate_soc(3.7) {
        Ok(soc) => println!("SOC: {:.1}%", soc),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Supported Battery Types
- LiPo (4.2V - 3.2V)
- LiFePO4 (3.65V - 3.0V)
- Li-Ion (4.2V - 3.3V)

## License
MIT
