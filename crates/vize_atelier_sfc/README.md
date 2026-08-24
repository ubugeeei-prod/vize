<h1 align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei-prod/vize/main/assets/crates/vize_atelier_sfc.svg" alt="vize_atelier_sfc logo" width="120" height="120" /><br>
  vize_atelier_sfc
</h1>

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_atelier_sfc` parses and compiles Vue Single File Components.

Tools that only need descriptor parsing should depend on
[`vize_croquis`](https://docs.rs/vize_croquis) and use its `sfc` module instead.
The full compiler re-exports that parser API, so existing `vize_atelier_sfc`
imports remain compatible.

## Highlights

- `.vue` descriptor parsing (`<template>`, `<script>`, `<script setup>`, `<style>`, custom blocks)
- Script compilation and binding metadata extraction
- Template compilation through DOM or Vapor backends
- Scoped CSS and style transforms powered by Lightning CSS
- Serialized CSS AST parse/print helpers for parser-backed tooling

## Key Entry Points

- `parse_sfc`
- `compile_sfc`
- `compile_css`
- `parse_css_ast`
- `print_css_ast`
- `compile_style_block`
- `SfcParseOptions`
- `SfcCompileOptions`

## Related Crates

- `vize_croquis::sfc` provides the parser and descriptor types without compiler backends
- `vize_atelier_dom` and `vize_atelier_vapor` compile template blocks
- `vize_croquis` and `vize_canon` consume emitted binding metadata and virtual TS
- `vize_vitrine` exposes this pipeline to Node.js and WASM consumers

## License

MIT
