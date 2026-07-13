# vize_atelier_jsx

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_atelier_jsx` owns the Vue JSX/TSX frontend. One OXC parse is projected into
owned JSX syntax and the independently requested Module, Croquis, Flow, Rendu,
or target products. Atlas is the execution substrate; JSX/TSX is not lowered
through Relief or a universal compiler IR.

## Highlights

- Source-faithful, parser-lifetime-free JSX/TSX syntax snapshots
- Module facts, Vue semantics, control/effect flow, and render intent selected by root
- Typed DOM, SSR, and Vapor output providers over Rendu
- Planning and execution share the same directive-prologue mode classification

## Key Entry Points

- `compile_jsx`
- `lower_source`
- `compile_to_vdom`
- `compile_to_vapor`
- `compile_to_ssr`
- `register_atlas_providers`

## License

MIT
