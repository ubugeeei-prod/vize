# vize_atelier_ssr

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_atelier_ssr` owns the frontend-independent Rendu-to-SSR backend. Its
graph provider consumes `RenduProduct` plus typed emission inputs and produces
`SsrOutputProduct` without parsing source.

## Highlights

- SSR-specific step configuration
- String-oriented code generation with SSR helpers
- Typed Atlas output through `SsrProvider`
- Legacy direct template compilation retained behind the `legacy` feature

## Key Entry Points

Graph backend:

- `register_atlas_provider`
- `SsrOutputProduct`
- `SsrProvider`

Legacy compatibility (`legacy` feature):

- `compile_ssr`
- `compile_ssr_with_options`
- `SsrCompilerOptions`
- `SsrCodegenResult`

## Related Crates

- `vize_rendu` owns the render intent consumed by the graph provider
- `vize_atelier_template`, SFC, and JSX routes produce Rendu for this backend
- `vize_atelier_core`, Relief, and Croquis are dependencies only of the legacy
  compatibility surface, not of the graph backend
- `@vizejs/vite-plugin` and Nuxt integration rely on this backend for SSR builds

## License

MIT
