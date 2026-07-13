---
title: Crates
---

# Crate Reference

> **⚠️ Work in Progress:** Vize is under active development. See the canonical
> [Rust crate support tiers](../stability.md#rust-crate-support-tiers) before depending on a public
> API.

Vize's Rust workspace is organized as focused crates. Each crate owns a representation, frontend,
backend, execution service, or tool product. Atlas composes those products on demand; the workspace
does not impose one syntax model on every source and consumer.

## Execution and Reusable Representations

| Crate             | Role                                                                                                                         |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `vize_carton`     | Shared allocator, strings, hash collections, flags, profiler, i18n, and DOM/tag utilities                                    |
| `vize_armature`   | Vue template tokenizer and parser                                                                                            |
| `vize_atlas`      | Execution substrate for typed providers, source revisions, planning, cache, invalidation, and traces; it owns no compiler IR |
| `vize_relief`     | Authored Vue-template nodes and locations, compiler errors, and compiler options                                             |
| `vize_croquis`    | Owned semantic contracts plus derived scopes, bindings, usage, and reactivity                                                |
| `vize_croquis_cf` | Opt-in lightweight project index plus full cross-file dependency/rule analysis                                               |
| `vize_flow`       | Frontend-neutral single-file control, data, effect graphs, and graph analyses                                                |
| `vize_module`     | Owned JS/TS module facts and OXC CFG projection shared by raw, SFC, JSX, and tools                                           |
| `vize_rendu`      | Owned, indexed, frontend-neutral render HIR and capabilities                                                                 |

## Compilation

| Crate                   | Role                                                                                                     |
| ----------------------- | -------------------------------------------------------------------------------------------------------- |
| `vize_atelier_core`     | Narrow legacy-compatible Vue-template transform/codegen lane; no graph or representation ownership       |
| `vize_atelier_dom`      | VDOM-oriented template compilation                                                                       |
| `vize_atelier_vapor`    | Vapor-mode template compilation                                                                          |
| `vize_atelier_ssr`      | Server-side rendering template compilation                                                               |
| `vize_atelier_template` | Independent raw Vue-template frontend producing Relief and Croquis, plus Flow or Rendu when requested    |
| `vize_atelier_sfc`      | `.vue` decomposition plus parse-once authored-script projections and script/template/style orchestration |
| `vize_atelier_jsx`      | JSX/TSX frontend producing owned syntax, Module, Croquis, Flow, or Rendu according to the requested root |

## Developer Tools

| Crate          | Role                                                                                 |
| -------------- | ------------------------------------------------------------------------------------ |
| `vize_patina`  | SFC, raw-template/HTML, JS/TS, and JSX/TSX lint products and diagnostic formatting   |
| `vize_glyph`   | Vue SFC formatter                                                                    |
| `vize_canon`   | Vue-aware type checking and virtual TypeScript generation                            |
| `vize_maestro` | Language Server Protocol implementation over one URI-keyed mutable Atlas compilation |
| `vize_musea`   | Musea art parsing, docs, palette generation, autogen, and VRT core                   |
| `vize_curator` | Local inspector payloads, graph/diff metadata, and profile reports                   |
| `vize_fresco`  | Terminal UI primitives used by TUI-oriented experiments                              |

## Distribution Layers

| Crate          | Role                                                                                          |
| -------------- | --------------------------------------------------------------------------------------------- |
| `vize_vitrine` | NAPI and WASM hosts for Atlas-backed compile, template, lint, typecheck, and cross-file roots |
| `vize`         | Rust-native CLI plus crate re-exports for docs                                                |

## Notes

- `vize_musea` is the Rust core for Musea art tooling. The gallery UI and dev-server workflow are
  provided by `@vizejs/vite-plugin-musea`.
- `vize_curator` is not published. It owns local developer artifacts such as inspector payloads,
  agent reports, cross-file graph metadata, and CLI profile report rendering. The low-level
  profiler remains in `vize_carton` because shared crates instrument their own hot paths.
- `vize_vitrine` is the bridge from Rust to JS. Packages such as `@vizejs/native` and
  `@vizejs/wasm` publish its binding surfaces. Vite, unplugin, Rspack, and Nuxt packages call those
  bindings; they are bundler hosts, not owners of Atlas products.
- `vize` is the full Rust CLI crate in the workspace. For v1 alpha, its public binary channel is
  GitHub Releases or Nix, while the npm `vize` package is the supported package-script entry point.

## Package Mapping

| Package / Command           | Main Rust root(s) and role                                                            |
| --------------------------- | ------------------------------------------------------------------------------------- |
| `vize build`                | `vize` requests SFC compile products and DOM/Vapor/SSR emitters                       |
| `vize fmt`                  | `vize` requests `vize_glyph` formatted-output products                                |
| `vize lint`                 | `vize` requests Patina document/module roots and optional `vize_croquis_cf` analysis  |
| `vize check`                | `vize` requests Canon typed-document roots                                            |
| `vize inspector`            | `vize_curator` requests `InspectorAgentReport` and per-source analysis products       |
| `vize lsp`                  | `vize_maestro` queries its persistent Atlas compilation                               |
| `@vizejs/native`            | `vize_vitrine` exposes SFC/JSX compile, raw-template compile, Patina, and Canon roots |
| `@vizejs/wasm`              | `vize_vitrine` exposes SFC/raw-template compile, Patina, Canon, and cross-file roots  |
| `@vizejs/vite-plugin`       | Bundler host over `@vizejs/native` SFC compile roots                                  |
| `@vizejs/unplugin`          | Rollup/webpack/esbuild host over native SFC and JSX compile roots                     |
| `@vizejs/rspack-plugin`     | Rspack host over the native SFC compile root                                          |
| `@vizejs/nuxt`              | Nuxt integration over the Vite host and shared configuration                          |
| `@vizejs/vite-plugin-musea` | `vize_musea` APIs exposed through `vize_vitrine`                                      |
| `@vizejs/musea-mcp-server`  | `vize_musea` APIs exposed through `vize_vitrine`                                      |
| `oxlint-plugin-vize`        | Patina diagnostics exposed through `vize_vitrine`                                     |
