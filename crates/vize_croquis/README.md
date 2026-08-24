<h1 align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei-prod/vize/main/assets/crates/vize_croquis.svg" alt="vize_croquis logo" width="120" height="120" /><br>
  vize_croquis
</h1>

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_croquis` is the semantic analysis layer for Vue templates and SFCs.

## Highlights

- Scope tracking for template and script bindings
- Binding metadata used by compilers and type checking
- Reactivity and macro analysis
- Cross-file and virtual TypeScript support modules
- Parse-only `.vue` descriptors without compiler backends

## Parse-only SFC Facade

Consumers that only need to inspect `.vue` descriptors can depend on
`vize_croquis` and use `vize_croquis::sfc::{parse_sfc, SfcDescriptor}` without
pulling in DOM, SSR, Vapor, code generation, or CSS transformation backends.

## Key Entry Points

- `Drawer`
- `DrawerOptions`
- `Croquis`
- `BindingMetadata`
- `sfc::parse_sfc`
- `sfc::SfcDescriptor`
- `ScopeChain`
- `SymbolTable`

## Related Crates

- `vize_armature` provides the parsed template tree
- `vize_atelier_dom`, `vize_atelier_vapor`, and `vize_atelier_ssr` consume binding metadata
- `vize_canon` and `vize_maestro` reuse the analysis layer for type-aware features

## License

MIT
