---
title: WASM Bindings
---

# WASM Bindings

Use `@vizejs/wasm` to compile, lint, or format Vue source inside a browser, for example in a
playground or an interactive documentation page. It runs the Rust compiler through WebAssembly
without requiring a compiler server.

[Install the package](#installation), initialize it once, then try the
[complete SFC compilation example](#compile-sfc). For a normal application build, follow the
[Vite Plugin guide](./vite-plugin.md). The WASM API is experimental; consult the
[package support tiers](../stability.md#package-support-tiers) when choosing an integration.

## Installation

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then add the package:

```bash
vp install @vizejs/wasm
```

## API

### Compile SFC

Compile a Vue Single File Component into JavaScript:

```javascript
import init, { compileSfc } from "@vizejs/wasm";

await init();

const result = compileSfc(
  `<template>
    <div>{{ msg }}</div>
  </template>

  <script setup lang="ts">
  const msg = ref('Hello Vize!')
  </script>`,
  { filename: "App.vue" },
);

console.log(result.script.code); // compiled <script> / <script setup>
console.log(result.template?.code); // compiled render function, when a template exists
console.log(result.css); // compiled styles, when styles exist
```

### Lint SFC

Run Vue-specific lint rules on an SFC:

```javascript
import init, { lintSfc } from "@vizejs/wasm";

await init();

const result = lintSfc(source, {
  filename: "App.vue",
  locale: "en", // 'en' | 'ja' | 'zh'
});

for (const diagnostic of result.diagnostics) {
  console.log(
    `${diagnostic.severity}: ${diagnostic.message} (line ${diagnostic.location.start.line})`,
  );
}
```

### Format SFC

Format a Vue SFC:

```javascript
import init, { formatSfc } from "@vizejs/wasm";

await init();

const formatted = formatSfc(source, { printWidth: 80 });

console.log(formatted.code);
```

## Initialization

The `init()` function must be called once before using any other API. It loads and instantiates the WebAssembly module:

```javascript
import init from "@vizejs/wasm";

// Basic initialization
await init();

// With custom WASM URL (useful for CDN or bundler setups)
await init("https://cdn.example.com/vize_vitrine_bg.wasm");
```

## Use Cases

### Playgrounds

Build interactive Vue compilation playgrounds that run entirely in the browser. The official [Vize Playground](https://vizejs.dev/play) uses the WASM bindings for real-time compilation:

```javascript
// React to editor changes and compile in real-time
editor.onChange((source) => {
  const result = compileSfc(source, {
    filename: "Playground.vue",
  });

  if (result.errors.length === 0) {
    preview.update({
      script: result.script.code,
      template: result.template?.code,
      css: result.css,
    });
  } else {
    diagnostics.show(result.errors);
  }
});
```

### Documentation

Embed live, editable Vue examples in your documentation:

```javascript
// Compile documentation examples on the fly
const examples = document.querySelectorAll("[data-vue-example]");
for (const el of examples) {
  const result = compileSfc(el.textContent, {
    filename: `example-${el.id}.vue`,
  });
  // Use result.script.code, result.template?.code, and result.css to mount it.
}
```

### Education

Create interactive compiler exploration tools that show the compilation output in real-time, helping developers understand how Vue templates are transformed.

### CI/CD

Use WASM bindings for lightweight compilation in environments where native binaries are not available (e.g., Cloudflare Workers, Deno Deploy, browser-based CI).

## Building from Source

```bash
# Install wasm-bindgen-cli
cargo install wasm-bindgen-cli

# Build WASM
cargo build --release -p vize_vitrine \
  --no-default-features \
  --features wasm \
  --target wasm32-unknown-unknown

# Generate JS bindings
wasm-bindgen \
  target/wasm32-unknown-unknown/release/vize_vitrine.wasm \
  --out-dir npm/wasm \
  --target web
```

## Internationalization

The `locale` option on `lintSfc` localizes lint diagnostics. The supported languages are:

| Code | Language          |
| ---- | ----------------- |
| `en` | English (default) |
| `ja` | Japanese (日本語) |
| `zh` | Chinese (中文)    |

Pass `locale` when calling `lintSfc`. `CompilerOptions` does not include `locale`; compilation and parsing use their existing diagnostic messages:

```javascript
const result = lintSfc(source, {
  filename: "App.vue",
  locale: "ja", // Lint messages in Japanese
});

console.log(result.diagnostics);
```

## Bundle Size

The WASM module includes the Vue compiler pipeline (parser, semantic analyzer, code generator). The compressed size depends on the build and generated artifact. After following the source-build recipe, run `gzip -c npm/wasm/vize_vitrine_bg.wasm | wc -c` to measure the gzip size of that exact module in bytes.

For interactive pages, load it after the page is ready or when the user first needs the compiler.

For production use, consider lazy-loading the WASM module:

```javascript
// Lazy-load the compiler only when needed
const compiler = await import("@vizejs/wasm");
await compiler.default(); // init()
const result = compiler.compileSfc(source, opts);
console.log(result.script.code, result.template?.code, result.css);
```

## Compiler option reference

### Compiler option compatibility

The `CompilerOptions` type is the supported option inventory for `compile`, `compileVapor`,
`parseTemplate`, and `compileSfc`. Unknown object keys are ignored at the JavaScript boundary and
are not compatibility promises. `vueParserQuirks` remains as a deprecated alias for
`templateSyntax: "quirks"`; an explicit `templateSyntax` always takes precedence. The shared Rust
field `experimentalServerScript` is reserved and is not exposed until a WASM compiler stage
implements it. Each facade ignores supported fields that do not apply to its compiler stage:
`bindingMetadata` only applies to direct template compilation. Runtime names apply to generated
VDOM modules and SFC client output (VDOM or Vapor); source maps apply to VDOM output, including the
template result returned by `compileSfc`. `outputMode` and `scriptExt` only apply to SFC compilation.
