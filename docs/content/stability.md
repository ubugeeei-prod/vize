---
title: Stability
description: Vize v1 alpha support tiers, compatibility promises, and experimental surfaces.
---

# Stability

Vize is moving toward a v1 alpha. The alpha contract is intentionally narrower than a stable v1
contract: it names the surfaces that should be usable by early adopters, while keeping room to
change internals and experimental integrations quickly. The full project is not yet a completely
production-ready toolchain; release decisions should use the
[production-readiness checklist](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/production-readiness.md).
Deprecation windows, SemVer rules, and release-line support are spelled out in the
[support policy](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/support-policy.md).

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
| Diagnostic codes listed in docs      | Should remain recognizable so suppressions and fix reports stay useful             |
| Published Rust crate APIs            | Follow the per-crate tier and deprecation contract below                           |
| Unexported Rust crate internals      | May change without migration support before v1 stable                              |
| Generated code and virtual TS output | May change when needed for correctness, compatibility, performance, or diagnostics |

## Runtime Support

The default Node.js floor for public npm runtime packages is Node 22, including
`oxlint-plugin-vize`. The Oxlint plugin declares `^22 || >= 24` so Node 22 and Node 24 or newer are
allowed while Node 23 stays outside the tested compatibility matrix.

The release workflow builds native packages for macOS, Linux, and Windows across x64 and arm64
where the package declares support. CI compatibility jobs cover the declared Node floor and the
current project Node version.

The full fresh-install smoke matrix (`.github/workflows/native-smoke.yml`) runs on a weekly
cadence and on demand, not on every PR push. It exercises the published package install path on
GitHub-hosted runners for linux-x64-gnu, linux-arm64-gnu, darwin-arm64, and win32-x64-msvc; the
remaining darwin-x64 and win32-arm64-msvc targets stay on architecture-specific hosted runners.
The matrix runs against Node 22 and Node 24. Release tags remain blocked by the release workflow's
tarball install smoke before npm packages publish. The runtime smoke checks `vize --version`,
`vize check`, `@vizejs/native` through both `require` and `import`, and a
`@vizejs/vite-plugin` `vite build` from installed tarballs.

Two declared Linux musl targets are not currently exercised by a hosted fresh-install runner.
They are covered by per-platform build artifacts plus the `@vizejs/native-*`
optional-dependency resolver until a containerized Alpine smoke can stage the matching native
tarball:

| Target           | Hosted runner gap                                                 | Compensating coverage                                              |
| ---------------- | ----------------------------------------------------------------- | ------------------------------------------------------------------ |
| linux-x64-musl   | No GitHub-hosted Alpine/musl VM is available as a native runner   | Build job emits the musl tarball; manual `node:alpine` smoke.      |
| linux-arm64-musl | Arm64 hosted runners are Ubuntu GNU, not Alpine/musl native hosts | Build job emits the arm64 musl tarball; manual Alpine arm64 smoke. |

Closing these gaps is tracked alongside [#493](https://github.com/ubugeeei-prod/vize/issues/493).

The minimum supported Rust version (MSRV) for the workspace is declared in `Cargo.toml` under
`[workspace.package].rust-version`. The development toolchain pinned by `rust-toolchain.toml`
may be the same version or newer. Before v1 stable the MSRV may move forward in any prerelease;
the move is called out in release notes when it changes. Downstream packagers should read
`rust-version` from a crate's `Cargo.toml` rather than infer it from the toolchain file.

## Package Support Tiers

| Tier                  | Packages                                                                                                      | Contract                                                                                       |
| --------------------- | ------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Alpha-supported       | `vize`, `@vizejs/native`, `@vizejs/vite-plugin`                                                               | Intended for early production trials with release-note-backed breaking changes.                |
| Compatibility preview | `@vizejs/unplugin`, `@vizejs/rspack-plugin`, `@vizejs/nuxt`, `@vizejs/nuxt-lint-config`, `@vizejs/musea-nuxt` | Expected to work for common host setups, but host-framework compatibility can move quickly.    |
| Experimental          | `oxlint-plugin-vize`, `@vizejs/vite-plugin-musea`, `@vizejs/musea-mcp-server`, `@vizejs/wasm`                 | Public packages, but APIs, commands, output, and workflow shape may change during alpha.       |
| Incubating            | `@vizejs/fresco`, `@vizejs/fresco-native`, editor extension packages                                          | Useful for development and feedback, but not yet part of the v1 alpha production-ready target. |

## Rust Crate Support Tiers

This table is the canonical compatibility contract for crates.io consumers. It covers every crate
whose Cargo metadata permits publishing, including crates temporarily deferred by the release
publisher while their first crates.io release is prepared. Private modules and implementation
details are not compatibility surfaces.

<!-- rust-crate-support:start -->

| Crate                | Tier                  | Intended audience                         | Public entrypoint                               | Removal / deprecation                  |
| -------------------- | --------------------- | ----------------------------------------- | ----------------------------------------------- | -------------------------------------- |
| `vize_carton`        | Alpha-supported       | Vize compiler and library authors         | `vize_carton::{Allocator, Box, FxHashMap}`      | One minor with `#[deprecated]`         |
| `vize_relief`        | Alpha-supported       | AST and compiler integration authors      | `vize_relief::{RootNode, CompilerOptions}`      | One minor with `#[deprecated]`         |
| `vize_armature`      | Alpha-supported       | Tools that parse Vue templates            | `vize_armature::{parse, Parser, Tokenizer}`     | One minor with `#[deprecated]`         |
| `vize_croquis`       | Compatibility preview | Semantic and type-aware tooling authors   | `vize_croquis::{Croquis, Drawer}`               | One minor with `#[deprecated]`         |
| `vize_croquis_cf`    | Experimental          | Opt-in whole-project analysis experiments | `vize_croquis_cf::CrossFileAnalyzer`            | No minimum; note breaks when practical |
| `vize_davinci`       | Experimental          | Davinci pipeline and dump-format authors  | `vize_davinci::{Folio, Diagnostic, NodeId}`      | No minimum; note breaks when practical |
| `vize_davinci_derive` | Experimental        | Davinci dump-format authors               | `vize_davinci_derive::Folio`                     | No minimum; note breaks when practical |
| `vize_doctor`        | Experimental          | Application health analyzer authors       | `vize_doctor::{DoctorFinding, FindingEvidence}` | No minimum; note breaks when practical |
| `vize_atelier_core`  | Alpha-supported       | Custom Vue compiler backend authors       | `vize_atelier_core::{transform, generate}`      | One minor with `#[deprecated]`         |
| `vize_atelier_dom`   | Alpha-supported       | VDOM compiler and bundler integrations    | `vize_atelier_dom::compile_template`            | One minor with `#[deprecated]`         |
| `vize_atelier_vapor` | Experimental          | Opt-in Vapor compiler integrations        | `vize_atelier_vapor::compile_vapor`             | No minimum; note breaks when practical |
| `vize_atelier_ssr`   | Compatibility preview | SSR and framework integration authors     | `vize_atelier_ssr::compile_ssr`                 | One minor with `#[deprecated]`         |
| `vize_atelier_sfc`   | Alpha-supported       | SFC tooling and bundler authors           | `vize_atelier_sfc::{parse_sfc, compile_sfc}`    | One minor with `#[deprecated]`         |
| `vize_atelier_jsx`   | Compatibility preview | JSX/TSX compiler and tooling authors      | `vize_atelier_jsx::{compile_jsx, lower_source}` | One minor with `#[deprecated]`         |
| `vize_marquette`     | Experimental          | Framework and target-adapter authors      | `vize_marquette::{ApplicationContract, Target}` | No minimum; note breaks when practical |
| `vize_musea`         | Experimental          | Musea gallery and documentation tools     | `vize_musea::{parse_art, transform_to_csf}`     | No minimum; note breaks when practical |
| `vize_fresco`        | Incubating            | TUI experiments                           | `vize_fresco::{RenderTree, LayoutEngine}`       | No minimum                             |
| `vize_canon`         | Compatibility preview | Type-checker and editor integrations      | `vize_canon::{type_check_sfc, TypeChecker}`     | One minor with `#[deprecated]`         |
| `vize_patina`        | Compatibility preview | Linter and Oxlint integrations            | `vize_patina::{lint, Linter}`                   | One minor with `#[deprecated]`         |
| `vize_s1`            | Experimental          | Lossless Vue-template tooling authors     | `vize_s1::{parse, SurfaceTree}`                  | No minimum; note breaks when practical |
| `vize_s1_to_s2`      | Experimental          | Vue lowering and compiler backend authors | `vize_s1_to_s2::{lower, emit_dom}`               | No minimum; note breaks when practical |
| `vize_s2`            | Experimental          | Dialect-neutral compiler IR authors       | `vize_s2::{op, verify, S2Folio}`                 | No minimum; note breaks when practical |

<!-- rust-crate-support:end -->

Each crate also records its tier in `package.metadata.vize.stability`. CI compares those Cargo
metadata values, this table, and the complete release-publisher crate set so adding, removing, or
reclassifying a publishable crate cannot silently change the contract.

### SemVer gate interpretation

`cargo-semver-checks` runs for the release publisher's crates that have resolvable registry
baselines. A crate waiting for its first publish, or blocked on one, joins that matrix once its
baseline is available. Until then, the metadata/table/release-list check still applies.

| Tier                                    | CI interpretation                                                                                                      |
| --------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Alpha-supported / Compatibility preview | An API break must be fixed or follow the support policy's deprecation window and carry a conventional breaking marker. |
| Experimental                            | The gate catches accidental drift; an intentional break may use a breaking marker without a deprecation window.        |
| Incubating                              | The same detection applies, but the entire API or crate may be replaced or removed in any release.                     |

The breaking markers recognized by CI are a `!` in the conventional change title or a
`BREAKING CHANGE:` footer. Passing the gate with either marker does not waive the deprecation
window for alpha-supported or compatibility-preview crates.

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
explicitly documents a different behavior. The release-blocking compiler, type-checking, runtime,
and Vite build surfaces are named in the
[Vue parity matrix](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/vue-parity-matrix.md).

For security handling, see the repository `SECURITY.md`. For contribution and fix workflow, see
`CONTRIBUTING.md`.
