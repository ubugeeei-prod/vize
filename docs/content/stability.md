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
Deprecation windows, SemVer rules, and release-line support are spelled out in the
[support policy](https://github.com/ubugeeei/vize/blob/main/docs/release/support-policy.md).

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

The full fresh-install smoke matrix (`.github/workflows/native-smoke.yml`) runs on a weekly
cadence and on demand, not on every PR push. It exercises the published package install path on
`ubuntu-latest` (linux-x64-gnu), `ubuntu-24.04-arm`
(linux-arm64-gnu), `macos-15-intel` (darwin-x64), `macos-latest` (darwin-arm64),
`windows-latest` (win32-x64-msvc), and `windows-11-arm` (win32-arm64-msvc)
against Node 22 and Node 24. Release tags remain blocked by the release workflow's tarball install
smoke before npm packages publish. The runtime smoke checks `vize --version`, `vize check`,
`@vizejs/native` through both `require` and `import`, and a `@vizejs/vite-plugin` `vite build`
from installed tarballs.

Two declared Linux musl targets are not currently exercised by a hosted fresh-install runner.
They are covered by per-platform build artifacts plus the `@vizejs/native-*`
optional-dependency resolver until a containerized Alpine smoke can stage the matching native
tarball:

| Target           | Hosted runner gap                                                 | Compensating coverage                                              |
| ---------------- | ----------------------------------------------------------------- | ------------------------------------------------------------------ |
| linux-x64-musl   | No GitHub-hosted Alpine/musl VM is available as a native runner   | Build job emits the musl tarball; manual `node:alpine` smoke.      |
| linux-arm64-musl | Arm64 hosted runners are Ubuntu GNU, not Alpine/musl native hosts | Build job emits the arm64 musl tarball; manual Alpine arm64 smoke. |

Closing these gaps is tracked alongside [#493](https://github.com/ubugeeei/vize/issues/493).

The minimum supported Rust version (MSRV) for the workspace is declared in `Cargo.toml` under
`[workspace.package].rust-version`. The development toolchain pinned by `rust-toolchain.toml`
may be the same version or newer. Before v1 stable the MSRV may move forward in any prerelease;
the move is called out in release notes when it changes. Downstream packagers should read
`rust-version` from a crate's `Cargo.toml` rather than infer it from the toolchain file.

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
explicitly documents a different behavior. The release-blocking compiler, type-checking, runtime,
and Vite build surfaces are named in the
[Vue parity matrix](https://github.com/ubugeeei/vize/blob/main/docs/release/vue-parity-matrix.md).

For security handling, see the repository `SECURITY.md`. For contribution and issue workflow, see
`CONTRIBUTING.md`.
