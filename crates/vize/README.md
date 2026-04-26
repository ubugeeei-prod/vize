# vize

`vize` is the Rust-native entry point for the Vize workspace.

It provides:

- the `vize` CLI binary (`build`, `fmt`, `lint`, `check`, `ready`, `upgrade`, `musea`, `lsp`, `ide`)
- a facade crate that re-exports the workspace crates for unified Rust docs

## Install

```bash
cargo install vize
```

## CLI

```bash
vize build src/**/*.vue
vize fmt --check src
vize lint --preset opinionated src
vize check --profile src
vize ready src
vize upgrade
vize lsp
```

`vize` defaults to `build` when no subcommand is provided.
`vize ready` runs `fmt --write`, `lint`, `check`, and `build` in order.
`vize upgrade` updates the installed Rust CLI through Cargo.

## Re-exported Crates

- `vize_carton` as `vize::carton`
- `vize_relief` as `vize::relief`
- `vize_armature` as `vize::armature`
- `vize_atelier_core` as `vize::atelier_core`
- `vize_atelier_dom` as `vize::atelier_dom`
- `vize_atelier_vapor` as `vize::atelier_vapor`
- `vize_atelier_sfc` as `vize::atelier_sfc`
- `vize_patina` as `vize::patina`
- `vize_canon` as `vize::canon`
- `vize_musea` as `vize::musea`
- `vize_maestro` as `vize::maestro`
- `vize_glyph` as `vize::glyph` when the `glyph` feature is enabled

## Related Crates

- `vize_atelier_sfc` powers the build pipeline.
- `vize_patina`, `vize_glyph`, and `vize_canon` power lint, format, and typecheck.
- `vize_maestro` powers `vize lsp`.

## License

MIT
