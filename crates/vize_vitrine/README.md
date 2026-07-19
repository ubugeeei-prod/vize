<h1 align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei-prod/vize/main/assets/crates/vize_vitrine.svg" alt="vize_vitrine logo" width="120" height="120" /><br>
  vize_vitrine
</h1>

`vize_vitrine` exposes Vize functionality to JavaScript through NAPI and WASM bindings.

## Highlights

- Template, SFC, lint, Musea, and typecheck bindings
- Shared FFI boundary types for both NAPI and WASM builds
- Optional `napi` and `wasm` feature gates

## Main Exports

- `CompilerOptions`
- `CompileResult`
- `type_check_sfc`
- `TypeCheckOptions`
- `TypeDiagnostic`

When the `napi` feature is enabled, the crate exposes bindings for template compilation, SFC
compilation, linting, Musea art tooling, and batch type checking. When the `wasm` feature is
enabled, equivalent browser-facing bindings are exported.

## Related Crates

- `npm/native` and `npm/wasm` publish this crate to JS users
- `@vizejs/vite-plugin`, `@vizejs/musea-mcp-server`, and other packages consume these bindings

## License

MIT
