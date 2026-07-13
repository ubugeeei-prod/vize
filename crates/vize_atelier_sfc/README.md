# vize_atelier_sfc

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_atelier_sfc` parses and compiles Vue Single File Components.

## Highlights

- `.vue` descriptor parsing (`<template>`, `<script>`, `<script setup>`, `<style>`, custom blocks)
- Parse-once authored-script projection: one live OXC `Program` supplies Module facts, Croquis analysis, and compiler preanalysis
- Source-shaped template products: Relief, Croquis, Flow, and Rendu are requested independently
- Typed DOM, SSR, or Vapor backend selection without owning raw-template compilation
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
- `register_atlas_providers`

## Related Crates

- `vize_atelier_dom`, `vize_atelier_ssr`, and `vize_atelier_vapor` emit target output from Rendu
- `vize_atelier_template` owns raw-template Relief/Croquis and optional Flow/Rendu roots
- `vize_croquis` consumes the SFC frontend's owned script and template projections
- `vize_canon` produces Virtual TS from the descriptor and Croquis, with Relief only for a template and Module only for a script; it does not fabricate Flow
- `vize_vitrine` exposes Atlas-backed SFC roots to Node.js and WASM consumers

## License

MIT
