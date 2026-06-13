---
title: Crates
---

# Crate Reference

> **⚠️ Work in Progress:** Vize is under active development. Crate APIs are still changing.

Vize's Rust workspace is organized around 18 primary crates. Each crate owns one reusable lane so
parsing, semantic analysis, code generation, linting, formatting, type checking, and
editor tooling can share the same syntax model.

## Foundation

| Crate           | Role                                                                                      |
| --------------- | ----------------------------------------------------------------------------------------- |
| `vize_carton`   | Shared allocator, strings, hash collections, flags, profiler, i18n, and DOM/tag utilities |
| `vize_relief`   | Shared Vue template AST, compiler errors, and compiler options                            |
| `vize_armature` | Vue template tokenizer and parser                                                         |
| `vize_croquis`  | Semantic analysis, scope tracking, binding metadata, reactivity, and virtual TS helpers   |

## Compilation

| Crate                | Role                                                          |
| -------------------- | ------------------------------------------------------------- |
| `vize_atelier_core`  | Shared transform lane and code generation infrastructure      |
| `vize_atelier_dom`   | VDOM-oriented template compilation                            |
| `vize_atelier_vapor` | Vapor-mode template compilation                               |
| `vize_atelier_ssr`   | Server-side rendering template compilation                    |
| `vize_atelier_sfc`   | `.vue` parsing plus script, template, and style orchestration |

## Developer Tools

| Crate          | Role                                                               |
| -------------- | ------------------------------------------------------------------ |
| `vize_patina`  | Vue SFC linter and diagnostic formatting                           |
| `vize_glyph`   | Vue SFC formatter                                                  |
| `vize_canon`   | Vue-aware type checking and virtual TypeScript generation          |
| `vize_maestro` | Language Server Protocol implementation                            |
| `vize_musea`   | Musea art parsing, docs, palette generation, autogen, and VRT core |
| `vize_curator` | Local inspector payloads, graph/diff metadata, and profile reports |
| `vize_fresco`  | Terminal UI primitives used by TUI-oriented experiments            |

## Distribution Layers

| Crate          | Role                                           |
| -------------- | ---------------------------------------------- |
| `vize_vitrine` | Shared NAPI and WASM bindings for JS consumers |
| `vize`         | Rust-native CLI plus crate re-exports for docs |

## Notes

- `vize_musea` is the Rust core for Musea art tooling. The gallery UI and dev-server workflow are
  provided by `@vizejs/vite-plugin-musea`.
- `vize_curator` is not published. It owns local developer artifacts such as inspector payloads,
  agent reports, cross-file graph metadata, and CLI profile report rendering. The low-level
  profiler remains in `vize_carton` because shared crates instrument their own hot paths.
- `vize_vitrine` is the bridge from Rust to JS. Packages such as `@vizejs/native` and
  `@vizejs/wasm` publish its bindings.
- `vize` is the full Rust CLI crate in the workspace. For v1 alpha, its public binary channel is
  GitHub Releases or Nix, while the npm `vize` package is the supported package-script entry point.

## Package Mapping

| Package / Command           | Main Rust crate(s)                                                                       |
| --------------------------- | ---------------------------------------------------------------------------------------- |
| `vize build`                | `vize`, `vize_atelier_sfc`, `vize_atelier_dom`, `vize_atelier_vapor`, `vize_atelier_ssr` |
| `vize fmt`                  | `vize`, `vize_glyph`                                                                     |
| `vize lint`                 | `vize`, `vize_patina`                                                                    |
| `vize check`                | `vize`, `vize_canon`                                                                     |
| `vize inspector`            | `vize`, `vize_curator`                                                                   |
| `vize lsp`                  | `vize`, `vize_maestro`                                                                   |
| `@vizejs/vite-plugin`       | `vize_vitrine`, `vize_atelier_sfc`                                                       |
| `@vizejs/native`            | `vize_vitrine`                                                                           |
| `@vizejs/wasm`              | `vize_vitrine`                                                                           |
| `@vizejs/vite-plugin-musea` | `vize_musea`, `vize_vitrine`                                                             |
| `@vizejs/musea-mcp-server`  | `vize_musea`, `vize_vitrine`                                                             |
| `oxlint-plugin-vize`        | `vize_patina`, `vize_vitrine`                                                            |
