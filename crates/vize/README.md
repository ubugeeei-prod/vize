# vize

`vize` is the Rust-native entry point for the Vize workspace.

It provides:

- the `vize` CLI binary (`build`, `fmt`, `lint`, `check`, `ready`, `upgrade`, `musea`, `lsp`, `ide`)
- a facade crate that re-exports its public tool and representation crates for unified Rust docs

## Install

For v1 alpha, the Rust CLI is distributed as GitHub release binaries and through the Nix entry
point. This crate is not published through crates.io yet.

```bash
nix run github:ubugeeei-prod/vize#vize -- --help
```

## CLI

```bash
vize build src/**/*.vue
vize fmt --check src
vize lint --preset opinionated src
vize check --profile src
vize ready src
vize lsp
```

`vize` defaults to `build` when no subcommand is provided.
`vize ready` runs `fmt --write`, `lint`, `check`, and `build` in order.
`vize upgrade` updates the npm package through Vite+ by default; use `--source cargo` only for
explicit local Cargo installs.

## Re-exported Crates

- `vize_carton` as `vize::carton`
- `vize_relief` as `vize::relief`
- `vize_armature` as `vize::armature`
- `vize_atelier_core` as `vize::atelier_core` for legacy template compatibility
- `vize_atelier_dom` as `vize::atelier_dom`
- `vize_atelier_ssr` as `vize::atelier_ssr`
- `vize_atelier_vapor` as `vize::atelier_vapor`
- `vize_atelier_template` as `vize::atelier_template`
- `vize_atelier_sfc` as `vize::atelier_sfc`
- `vize_atelier_jsx` as `vize::atelier_jsx`
- `vize_rendu` as `vize::rendu`
- `vize_flow` as `vize::flow`
- `vize_module` as `vize::module`
- `vize_croquis` as `vize::croquis`
- `vize_croquis_cf` as `vize::croquis_cf`
- `vize_patina` as `vize::patina`
- `vize_canon` as `vize::canon`
- `vize_musea` as `vize::musea`
- `vize_maestro` as `vize::maestro`
- `vize_glyph` as `vize::glyph` when the `glyph` feature is enabled

## Related Crates

- `vize_atelier_template` powers standalone template compilation without synthetic SFCs.
- `vize_atelier_sfc` powers the SFC build pipeline.
- `vize_atelier_jsx` owns the JSX/TSX frontend.
- `vize_rendu` carries render intent to the DOM, SSR, and Vapor backends; optional
  `vize_flow` products remain independent.
- `vize_patina`, `vize_glyph`, and `vize_canon` power lint, format, and typecheck.
- `vize_maestro` powers `vize lsp`.

The CLI assembles these products with `vize_atlas`, which owns source identity,
planning, caching, and invalidation rather than a shared AST or compiler IR.

## License

MIT
