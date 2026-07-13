# vize_atelier_jsx

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_atelier_jsx` owns the Vue JSX/TSX frontend. One OXC parse is projected into
owned JSX syntax and the independently requested Module, Croquis, Flow, Rendu,
or compile-recipe products. Atlas is the execution substrate; JSX/TSX is not lowered
through Relief or a universal compiler IR.

## Highlights

- Source-faithful, parser-lifetime-free JSX/TSX syntax snapshots
- Module facts, Vue semantics, control/effect flow, and render intent selected by root
- Frontend-owned registrar; application hosts explicitly compose peer DOM, SSR, and Vapor backends
- Planning and execution share the same directive-prologue mode classification

## Key Entry Points

- `compile_jsx`
- `lower_source`
- `compile_to_vdom`
- `compile_to_vapor`
- `compile_to_ssr`
- `register_atlas_providers`

`register_atlas_providers` never invokes another frontend, Croquis projection,
or render-backend registrar. Compile hosts add only the peer backends they
offer; an unrequested backend is not planned or executed.

## License

MIT
