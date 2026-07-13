# vize_atelier_core

`vize_atelier_core` contains only the shared Vue-template transform lane and
legacy emission helpers used by the DOM, Vapor, and SSR compilers.

It is not the owner of the compiler foundation. `vize_atlas` owns typed graph
execution, `vize_relief` owns Vue-template syntax, `vize_croquis` owns semantic
contracts, `vize_flow` owns CFG/data/effect graphs, and `vize_rendu` owns the
frontend-neutral render HIR. Atelier Core does not depend on Atlas or Rendu.

The root-level Relief AST, Armature parser, and Carton allocator re-exports are
compatibility aliases for existing downstream users. They are not the canonical
workspace API and do not make Atelier Core the owner of those representations.
Production workspace crates import each owner directly:

| Concern | Owning crate |
| --- | --- |
| Vue-template syntax, locations, and compiler options | `vize_relief` |
| Vue-template tokenization and parsing | `vize_armature` |
| Arena allocation and shared utility primitives | `vize_carton` |
| Semantic identity, scopes, bindings, and usage | `vize_croquis` |
| Shared legacy transform and emission mechanics | `vize_atelier_core` |

## Highlights

- Core lane and step APIs
- Shared Vue template code generation
- Runtime helper resolution
- Compatibility-only re-exports for Relief, Armature, and Carton APIs

Atelier Core does not register Atlas products, own syntax or semantic
snapshots, select compiler frontends or backends, or orchestrate production
tool requests. Those responsibilities stay with their peer crates and host
recipes.

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
