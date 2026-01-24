# keystone-wasm

WASM bindings for Keystone financial computation.

## Installation

```bash
npm install @dijkstra-keystone/wasm
```

## Usage

```javascript
import init, * as keystone from '@dijkstra-keystone/wasm';

async function main() {
  await init();

  // Arithmetic
  keystone.add("100.50", "49.50");     // "150"
  keystone.multiply("99.99", "1.08");  // "107.9892"
  keystone.divide("100", "3");         // "33.333..."

  // Rounding
  keystone.round("123.456", 2, "half_up");  // "123.46"

  // Financial
  keystone.compound_interest("10000", "0.05", 12, 5);
  keystone.future_value("10000", "0.07", 10);

  // Risk
  keystone.health_factor("10000", "5000", "0.80");  // "1.6"
  keystone.liquidation_price("5", "10000", "0.80");
}
```

## Features

- Full precision-core arithmetic
- All financial-calc functions
- All risk-metrics functions
- ~97KB optimized bundle
- Deterministic cross-platform results

## API

### Arithmetic
- `add(a, b)`, `subtract(a, b)`, `multiply(a, b)`, `divide(a, b)`
- `abs(value)`, `compare(a, b)`, `min(a, b)`, `max(a, b)`

### Rounding
- `round(value, dp, mode)` - Modes: "half_even", "half_up", "down", "up", etc.
- `floor(value)`, `ceil(value)`, `trunc(value)`

### Financial
- `simple_interest`, `compound_interest`, `effective_annual_rate`
- `future_value`, `present_value`, `net_present_value`
- `percentage_of`, `percentage_change`

### Risk
- `health_factor`, `is_healthy`, `collateral_ratio`
- `liquidation_price`, `max_borrowable`
- `loan_to_value`, `utilization_rate`

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
