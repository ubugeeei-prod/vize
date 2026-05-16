# vize_atelier_dom

`vize_atelier_dom` compiles Vue templates for the DOM runtime.

## Highlights

- DOM-aware transforms for directives such as `v-model`, `v-show`, `v-text`, `v-html`, and `v-on`
- Platform-specific namespace handling for HTML, SVG, and MathML
- Shared parser and transform pipeline from `vize_atelier_core`

## Key Entry Points

- `compile_template`
- `compile_template_with_options`
- `DomCompilerOptions`

## Related Crates

- `vize_atelier_core` provides shared transforms and codegen
- `vize_croquis` can be passed in through compiler options for semantic context
- `vize_atelier_sfc` uses this crate for standard template compilation

## License

MIT
