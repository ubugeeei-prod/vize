# vize_atelier_dom

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_atelier_dom` owns the frontend-independent Rendu-to-DOM/VDOM backend.
Its graph provider consumes `RenduProduct` plus typed emission inputs and
produces `DomOutputProduct` without parsing source.

## Highlights

- DOM-aware steps for directives such as `v-model`, `v-show`, `v-text`, `v-html`, and `v-on`
- Platform-specific namespace handling for HTML, SVG, and MathML
- Typed Atlas output through `DomProvider`
- Legacy direct template compilation retained behind the `legacy` feature

## Key Entry Points

Graph backend:

- `register_atlas_provider`
- `DomOutputProduct`
- `DomProvider`

Legacy compatibility (`legacy` feature):

- `compile_template`
- `compile_template_with_options`
- `DomCompilerOptions`

## Related Crates

- `vize_rendu` owns the render intent consumed by the graph provider
- `vize_atelier_template`, SFC, and JSX routes produce Rendu for this backend
- `vize_atelier_core`, Relief, and Croquis are dependencies only of the legacy
  compatibility surface, not of the graph backend

## License

MIT
