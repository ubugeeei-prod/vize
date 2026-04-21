# vize_atelier_sfc

`vize_atelier_sfc` parses and compiles Vue Single File Components.

## Highlights

- `.vue` descriptor parsing (`<template>`, `<script>`, `<script setup>`, `<style>`, custom blocks)
- Script compilation and binding metadata extraction
- Template compilation through DOM or Vapor backends
- Scoped CSS and style transforms powered by Lightning CSS

## Key Entry Points

- `parse_sfc`
- `compile_sfc`
- `compile_css`
- `compile_style_block`
- `SfcParseOptions`
- `SfcCompileOptions`

## Related Crates

- `vize_atelier_dom` and `vize_atelier_vapor` compile template blocks
- `vize_croquis` and `vize_canon` consume emitted binding metadata and virtual TS
- `vize_vitrine` exposes this pipeline to Node.js and WASM consumers

## License

MIT
