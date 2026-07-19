<h1 align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei-prod/vize/main/assets/crates/vize_atelier_dom.svg" alt="vize_atelier_dom logo" width="120" height="120" /><br>
  vize_atelier_dom
</h1>

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_atelier_dom` compiles Vue templates for the DOM runtime.

## Highlights

- DOM-aware steps for directives such as `v-model`, `v-show`, `v-text`, `v-html`, and `v-on`
- Platform-specific namespace handling for HTML, SVG, and MathML
- Shared parser and transform lane from `vize_atelier_core`

## Key Entry Points

- `compile_template`
- `compile_template_with_options`
- `DomCompilerOptions`

## Related Crates

- `vize_atelier_core` provides shared steps and codegen
- `vize_croquis` can be passed in through compiler options for semantic context
- `vize_atelier_sfc` uses this crate for standard template compilation

## License

MIT
