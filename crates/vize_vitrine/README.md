# vize_vitrine

`vize_vitrine` exposes source-shaped Vize products to JavaScript through NAPI
and WASM bindings. Each stateless call, or shared batch call where supported,
assembles the Atlas roots required by that binding instead of routing every
input through one frontend.

## Highlights

- Raw-template, SFC, and JSX/TSX compilation with independently selected outputs
- Patina lint, Canon typecheck, Inspector, formatting, and Musea bindings where
  supported by the selected host
- WASM Croquis and cross-file analysis bindings
- Shared FFI boundary types for both NAPI and WASM builds
- Optional `napi` and `wasm` feature gates

## Main Exports

- Raw-template `compile` and `compileVapor`
- SFC compilation and batch compilation
- JSX/TSX `compileJsx`
- Lint, typecheck, formatting, Inspector, and Musea operations exposed by the
  selected host

The NAPI surface includes raw-template, SFC, and JSX/TSX compilation, lint and
fixes, formatting, Canon type checking, Inspector, and Musea operations. The
WASM surface includes browser-facing raw-template, SFC, and JSX/TSX compilation,
Patina lint, Canon type checking, Croquis single-file and cross-file analysis,
Inspector, Musea, and formatting when Glyph is enabled. These surfaces share FFI
types where practical, but they are not assumed to export identical operations.

## Related Crates

- `npm/native` and `npm/wasm` publish this crate to JS users
- `@vizejs/vite-plugin`, `@vizejs/musea-mcp-server`, and other packages consume these bindings

## License

MIT
