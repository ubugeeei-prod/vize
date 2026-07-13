# vize_atelier_vapor

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_atelier_vapor` owns the frontend-independent Rendu-to-Vapor backend. Its
graph providers consume `RenduProduct` plus typed emission inputs, produce an
owned Vapor plan, and emit `VaporOutputProduct` without parsing source.

## Highlights

- Owned Vapor planning from frontend-neutral render intent
- Typed Atlas planning and output through `VaporProvider` and
  `VaporOutputProvider`
- Legacy Vapor-specific IR, code-generation helpers, and direct template
  compilation retained behind the `legacy` feature

## Key Entry Points

Graph backend:

- `register_atlas_provider`
- `VaporPlanProduct`
- `VaporOutputProduct`

Legacy compatibility (`legacy` feature):

- `compile_vapor`
- `transform_to_ir`
- `generate_vapor`
- `VaporCompilerOptions`

## Related Crates

- `vize_rendu` owns the render intent consumed by the graph provider
- `vize_atelier_template`, SFC, and JSX routes produce Rendu for this backend
- `vize_atelier_core`, Relief, and Croquis are dependencies only of the legacy
  compatibility surface, not of the graph backend
- `vize_patina` includes Vapor-oriented lint rules that align with this backend

## License

MIT
