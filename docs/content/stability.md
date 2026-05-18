---
title: Stability
description: Vize v1 alpha support tiers, compatibility promises, and experimental surfaces.
---

# Stability

Vize is moving toward a v1 alpha. The alpha contract is intentionally narrower than a stable v1
contract: it names the surfaces that should be usable by early adopters, while keeping room to
change internals and experimental integrations quickly. The full project is not yet a completely
production-ready toolchain; release decisions should use the
[production-readiness checklist](https://github.com/ubugeeei/vize/blob/main/docs/release/production-readiness.md).

## Versioning Contract

Before v1 stable, any prerelease can include breaking changes. Vize still treats breaking changes as
release-note material, especially when they affect package entrypoints, CLI flags, config fields,
diagnostic codes, or generated output.

The v1 alpha line uses these rules:

| Surface                              | Alpha expectation                                                                  |
| ------------------------------------ | ---------------------------------------------------------------------------------- |
| Published package names              | Should remain available or ship with migration notes                               |
| Documented CLI commands and flags    | Should avoid silent behavior changes                                               |
| Documented config fields             | Should keep names and value shapes stable unless release notes call out a change   |
| Diagnostic codes listed in docs      | Should remain recognizable so suppressions and issue reports stay useful           |
| Rust crate internals                 | May change without migration support before v1 stable                              |
| Generated code and virtual TS output | May change when needed for correctness, compatibility, performance, or diagnostics |

## Runtime Support

The default Node.js floor for public npm runtime packages is Node 22. `oxlint-plugin-vize` is the
exception and requires Node 24 because it follows Oxlint's JavaScript plugin runtime.

The release workflow builds native packages for macOS, Linux, and Windows across x64 and arm64
where the package declares support. CI compatibility jobs cover the declared Node floor and the
current project Node version.

## Package Support Tiers

| Tier                  | Packages                                                                                      | Contract                                                                                       |
| --------------------- | --------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Alpha-supported       | `vize`, `@vizejs/native`, `@vizejs/vite-plugin`                                               | Intended for early production trials with release-note-backed breaking changes.                |
| Compatibility preview | `@vizejs/unplugin`, `@vizejs/rspack-plugin`, `@vizejs/nuxt`, `@vizejs/musea-nuxt`             | Expected to work for common host setups, but host-framework compatibility can move quickly.    |
| Experimental          | `oxlint-plugin-vize`, `@vizejs/vite-plugin-musea`, `@vizejs/musea-mcp-server`, `@vizejs/wasm` | Public packages, but APIs, commands, output, and workflow shape may change during alpha.       |
| Incubating            | `@vizejs/fresco`, `@vizejs/fresco-native`, editor extension packages                          | Useful for development and feedback, but not yet part of the v1 alpha production-ready target. |

## What Counts as Stable Enough for Alpha

A package or command can move into the alpha-supported tier when it has:

- documented install and usage paths
- CI coverage for package build, install, and supported Node runtime
- release smoke coverage for published entrypoints
- a clear owner for regressions and compatibility reports
- known unsupported behavior documented in the relevant guide

## What Is Not Promised Yet

The alpha does not promise full compatibility with every Vue compiler edge case, every package
manager layout, every editor capability, or every framework integration. When Vize disagrees with
official Vue tooling, treat the official output as the compatibility baseline unless a Vize guide
explicitly documents a different behavior.

For security handling, see the repository `SECURITY.md`. For contribution and issue workflow, see
`CONTRIBUTING.md`.
