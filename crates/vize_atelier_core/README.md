# vize_atelier_core

`vize_atelier_core` contains the shared transform lane and code generation infrastructure used by the
DOM, Vapor, and SSR compilers.

It is not the owner of the compiler foundation. `vize_atlas` owns typed graph
execution, `vize_relief` owns Vue-template syntax, `vize_croquis` owns semantic
contracts, `vize_flow` owns CFG/data/effect graphs, and `vize_rendu` owns the
frontend-neutral render HIR. Atelier Core does not depend on Atlas or Rendu.

## Highlights

- Core lane and step APIs
- Shared Vue template code generation
- Runtime helper resolution
- Re-exports for the Relief AST and Armature parser APIs

## Key Entry Points

- `lane::transform`
- `steps`
- `generate`
- `RuntimeHelpers`
- `lane::TransformContext`
- `lane::DirectiveTransform`
- `lane::NodeTransform`

## Related Crates

- `vize_armature` parses templates
- `vize_atlas` plans and caches independently registered products
- `vize_relief` owns syntax; `vize_croquis` derives semantic relationships
- `vize_rendu` exposes owned, indexed render HIR
- `vize_atelier_dom`, `vize_atelier_vapor`, and `vize_atelier_ssr` provide platform-specific backends
- `vize_atelier_sfc` orchestrates full `.vue` compilation on top of these building blocks

## License

MIT
