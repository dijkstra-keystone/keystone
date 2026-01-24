# Installation

## Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
precision-core = "0.1"
financial-calc = "0.1"  # optional
risk-metrics = "0.1"    # optional
```

For `no_std` environments:

```toml
[dependencies]
precision-core = { version = "0.1", default-features = false }
```

## JavaScript / TypeScript

```bash
npm install @dijkstra-keystone/wasm
```

Or via CDN:

```html
<script type="module">
  import init, * as keystone from 'https://unpkg.com/@dijkstra-keystone/wasm';
  await init();
  console.log(keystone.add("0.1", "0.2")); // "0.3"
</script>
```

## Building from Source

```bash
git clone https://github.com/dijkstra-keystone/keystone
cd keystone
cargo build --release
```

For WASM:

```bash
cargo build --target wasm32-unknown-unknown --release -p wasm-bindings
```
