# WASM API Reference

Complete API reference for the WebAssembly bindings.

## Arithmetic Operations

### add(a, b)

Add two decimal values.

```javascript
add("100.50", "49.50")  // "150"
add("-10", "5")         // "-5"
```

**Parameters:**
- `a: string` - First operand
- `b: string` - Second operand

**Returns:** `string` - Sum

**Throws:** On overflow or invalid input

---

### subtract(a, b)

Subtract b from a.

```javascript
subtract("100", "30")   // "70"
subtract("10", "25")    // "-15"
```

**Parameters:**
- `a: string` - Minuend
- `b: string` - Subtrahend

**Returns:** `string` - Difference

---

### multiply(a, b)

Multiply two decimal values.

```javascript
multiply("25", "4")     // "100"
multiply("0.1", "0.1")  // "0.01"
```

**Parameters:**
- `a: string` - First factor
- `b: string` - Second factor

**Returns:** `string` - Product

**Throws:** On overflow

---

### divide(a, b)

Divide a by b.

```javascript
divide("100", "4")      // "25"
divide("10", "3")       // "3.333..."
```

**Parameters:**
- `a: string` - Dividend
- `b: string` - Divisor

**Returns:** `string` - Quotient

**Throws:** On division by zero

---

### remainder(a, b)

Calculate remainder of a divided by b.

```javascript
remainder("10", "3")    // "1"
remainder("7.5", "2")   // "1.5"
```

---

## Comparison Operations

### compare(a, b)

Compare two decimal values.

```javascript
compare("100", "200")   // -1
compare("200", "200")   // 0
compare("300", "200")   // 1
```

**Returns:** `number` - -1 if a < b, 0 if equal, 1 if a > b

---

### min(a, b) / max(a, b)

Return minimum or maximum of two values.

```javascript
min("100", "200")       // "100"
max("100", "200")       // "200"
```

---

## Rounding Operations

### round(value, dp, mode)

Round to specified decimal places.

```javascript
round("123.456", 2, "half_up")     // "123.46"
round("123.445", 2, "half_even")   // "123.44"
```

**Parameters:**
- `value: string` - Value to round
- `dp: number` - Decimal places (0-28)
- `mode: string` - Rounding mode

**Rounding Modes:**
| Mode | Description |
|------|-------------|
| `"half_even"` or `"bankers"` | Ties round to nearest even |
| `"half_up"` | Ties round away from zero |
| `"half_down"` | Ties round toward zero |
| `"up"` | Always toward +infinity |
| `"down"` | Always toward -infinity |
| `"toward_zero"` or `"truncate"` | Truncate |
| `"away_from_zero"` | Away from zero |

---

### floor(value) / ceil(value) / trunc(value)

Convenience rounding functions.

```javascript
floor("2.7")    // "2"
floor("-2.7")   // "-3"
ceil("2.3")     // "3"
ceil("-2.3")    // "-2"
trunc("2.9")    // "2"
trunc("-2.9")   // "-2"
```

---

## Properties

### abs(value)

Absolute value.

```javascript
abs("-42")      // "42"
abs("42")       // "42"
```

---

### is_zero(value) / is_positive(value) / is_negative(value)

Boolean checks.

```javascript
is_zero("0.00")        // true
is_positive("5")       // true
is_negative("-5")      // true
```

---

### scale(value)

Get the scale (decimal places).

```javascript
scale("123.45")        // 2
scale("100")           // 0
scale("1.000")         // 3
```

---

### normalize(value)

Remove trailing zeros.

```javascript
normalize("100.00")    // "100"
normalize("1.50")      // "1.5"
```

---

## Financial Functions

### simple_interest(principal, rate, time)

Calculate simple interest.

```javascript
simple_interest("10000", "0.05", "3")  // "1500"
```

---

### compound_interest(principal, rate, periods_per_year, years)

Calculate compound interest.

```javascript
compound_interest("10000", "0.05", 12, 5)  // Total interest
```

---

### future_value(present_value, rate, periods)

Calculate future value.

```javascript
future_value("10000", "0.07", 10)  // ~"19671.51"
```

---

### present_value(future_value, rate, periods)

Calculate present value.

```javascript
present_value("19672", "0.07", 10)  // ~"10000"
```

---

### percentage_of(value, percent)

Calculate percentage of a value.

```javascript
percentage_of("200", "0.15")  // "30" (15% of 200)
```

---

### percentage_change(old_value, new_value)

Calculate percentage change.

```javascript
percentage_change("100", "125")  // "0.25" (25% increase)
```

---

## Risk Functions

### health_factor(collateral, debt, threshold)

Calculate lending position health factor.

```javascript
health_factor("10000", "5000", "0.80")  // "1.6"
```

---

### is_healthy(collateral, debt, threshold)

Check if position is healthy (HF >= 1).

```javascript
is_healthy("10000", "5000", "0.80")  // true
```

---

### liquidation_price(collateral_amount, debt, threshold)

Calculate liquidation price.

```javascript
liquidation_price("5", "10000", "0.80")  // "2500"
```

---

### max_borrowable(collateral, threshold, min_health_factor)

Calculate maximum safe debt.

```javascript
max_borrowable("100000", "0.80", "1.5")  // "53333.33..."
```

---

## Tolerance Functions

### approx_eq(a, b, tolerance)

Check if values are approximately equal.

```javascript
approx_eq("100.00", "100.01", "0.02")  // true
```

---

### within_percentage(a, b, percent)

Check if values are within percentage of each other.

```javascript
within_percentage("102", "100", "5")  // true (within 5%)
```

---

### within_basis_points(a, b, bps)

Check if values are within basis points.

```javascript
within_basis_points("1.0010", "1.0000", "15")  // true (within 15 bps)
```

---

## Constants

```javascript
ZERO        // "0"
ONE         // "1"
ONE_HUNDRED // "100"
MAX         // Maximum representable value
MIN         // Minimum representable value
```

---

## Error Types

All functions throw JavaScript errors on failure:

```javascript
try {
  divide("1", "0");
} catch (e) {
  // e.message contains error description
  switch (true) {
    case e.message.includes("division by zero"):
      // Handle division by zero
      break;
    case e.message.includes("overflow"):
      // Handle overflow
      break;
    case e.message.includes("invalid"):
      // Handle parse error
      break;
  }
}
```
