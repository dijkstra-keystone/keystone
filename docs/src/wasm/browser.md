# Browser Integration

Using Keystone in web applications via WebAssembly.

## Installation

```bash
npm install @dijkstra-keystone/wasm
# or
yarn add @dijkstra-keystone/wasm
```

## Setup

### ES Modules

```javascript
import init, * as keystone from '@dijkstra-keystone/wasm';

async function setup() {
  await init();
  // Now ready to use
  const result = keystone.add("100.50", "49.50");
  console.log(result); // "150"
}
```

### Bundlers (Webpack, Vite, etc.)

Most bundlers handle WASM automatically:

```javascript
// vite.config.js
export default {
  optimizeDeps: {
    exclude: ['@dijkstra-keystone/wasm']
  }
}
```

```javascript
// In your code
import * as keystone from '@dijkstra-keystone/wasm';

const result = keystone.multiply("99.99", "1.0825");
```

### Script Tag

```html
<script type="module">
  import init, * as keystone from './keystone_wasm.js';

  init().then(() => {
    document.getElementById('result').textContent =
      keystone.add("1.1", "2.2");
  });
</script>
```

## Basic Usage

All functions accept string representations of decimals:

```javascript
import * as keystone from '@dijkstra-keystone/wasm';

// Arithmetic
keystone.add("100", "50");           // "150"
keystone.subtract("100", "30");      // "70"
keystone.multiply("25", "4");        // "100"
keystone.divide("100", "3");         // "33.333..."

// Comparison
keystone.compare("100", "200");      // -1 (less than)
keystone.compare("200", "200");      // 0 (equal)
keystone.compare("300", "200");      // 1 (greater than)

// Properties
keystone.abs("-42");                 // "42"
keystone.is_zero("0.00");            // true
keystone.is_negative("-5");          // true
```

## Rounding

```javascript
import { round } from '@dijkstra-keystone/wasm';

// Round to 2 decimal places
round("123.456", 2, "half_even");   // "123.46" (banker's)
round("123.456", 2, "half_up");     // "123.46"
round("123.455", 2, "half_up");     // "123.46"
round("123.454", 2, "half_up");     // "123.45"

// Available modes
round("2.5", 0, "half_even");       // "2" (ties to even)
round("2.5", 0, "half_up");         // "3" (ties away from zero)
round("2.5", 0, "down");            // "2" (toward -infinity)
round("2.5", 0, "up");              // "3" (toward +infinity)
round("2.5", 0, "toward_zero");     // "2" (truncate)
round("2.5", 0, "away_from_zero");  // "3"
```

## Financial Functions

```javascript
import * as keystone from '@dijkstra-keystone/wasm';

// Simple interest
keystone.simple_interest("10000", "0.05", "3");
// 10000 * 0.05 * 3 = "1500"

// Compound interest (principal, rate, periods_per_year, years)
keystone.compound_interest("10000", "0.05", 12, 5);
// Returns total interest earned

// Future value
keystone.future_value("10000", "0.07", 10);
// 10000 * (1.07)^10

// Present value
keystone.present_value("19672", "0.07", 10);
// Discounts future value to present
```

## Risk Metrics

```javascript
import * as keystone from '@dijkstra-keystone/wasm';

// Health factor
keystone.health_factor("10000", "5000", "0.80");
// (10000 * 0.80) / 5000 = "1.6"

// Is position healthy?
keystone.is_healthy("10000", "5000", "0.80");  // true

// Liquidation price
keystone.liquidation_price("5", "10000", "0.80");
// Price per unit at which position becomes liquidatable
```

## Error Handling

Functions throw on error:

```javascript
import * as keystone from '@dijkstra-keystone/wasm';

try {
  const result = keystone.divide("100", "0");
} catch (e) {
  console.error(e.message);  // "division by zero"
}

// Or use optional chaining pattern
function safeDivide(a, b) {
  try {
    return keystone.divide(a, b);
  } catch {
    return null;
  }
}
```

## React Integration

```jsx
import { useState, useEffect } from 'react';
import init, * as keystone from '@dijkstra-keystone/wasm';

function Calculator() {
  const [ready, setReady] = useState(false);
  const [result, setResult] = useState('');

  useEffect(() => {
    init().then(() => setReady(true));
  }, []);

  const calculate = (a, b) => {
    if (!ready) return;
    try {
      setResult(keystone.add(a, b));
    } catch (e) {
      setResult(`Error: ${e.message}`);
    }
  };

  return (
    <div>
      {ready ? (
        <button onClick={() => calculate("100.50", "49.50")}>
          Add
        </button>
      ) : (
        <span>Loading...</span>
      )}
      <div>Result: {result}</div>
    </div>
  );
}
```

## Vue Integration

```vue
<script setup>
import { ref, onMounted } from 'vue';
import init, * as keystone from '@dijkstra-keystone/wasm';

const ready = ref(false);
const result = ref('');

onMounted(async () => {
  await init();
  ready.value = true;
});

function calculate() {
  result.value = keystone.multiply("99.99", "1.0825");
}
</script>

<template>
  <button v-if="ready" @click="calculate">Calculate</button>
  <span v-else>Loading...</span>
  <div>{{ result }}</div>
</template>
```

## Performance Tips

1. **Batch operations**: Minimize JS-WASM boundary crossings
2. **Reuse results**: Store intermediate values in JS when possible
3. **Async init**: Initialize WASM during app startup, not on-demand

```javascript
// Pre-compute values
const TAX_RATE = "0.0825";
const DISCOUNT = "0.15";

function calculateTotal(items) {
  let subtotal = "0";
  for (const item of items) {
    subtotal = keystone.add(subtotal, item.price);
  }

  const discount = keystone.multiply(subtotal, DISCOUNT);
  const afterDiscount = keystone.subtract(subtotal, discount);
  const tax = keystone.multiply(afterDiscount, TAX_RATE);

  return keystone.add(afterDiscount, tax);
}
```
