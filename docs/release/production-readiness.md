# Production Readiness

Use this page to answer whether Vize is ready for production use. The current answer is scoped:
Vize can support early production trials for the alpha-supported package set, but the whole
repository is not yet a stable, production-ready toolchain.

## Current Support Scope

| Scope                          | Status          | Notes                                                                                                                                                                |
| ------------------------------ | --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `vize`                         | Alpha-supported | Suitable for early production trials when release notes and known limitations are reviewed.                                                                          |
| `@vizejs/native`               | Alpha-supported | Native package support depends on the release native smoke matrix and npm install smoke checks.                                                                      |
| `@vizejs/vite-plugin`          | Alpha-supported | Validate against the target app before rollout, especially for non-trivial Vue compiler behavior.                                                                    |
| Compatibility preview packages | Preview         | `@vizejs/unplugin`, `@vizejs/rspack-plugin`, `@vizejs/nuxt`, and `@vizejs/musea-nuxt` need host-framework validation before production use.                          |
| Experimental packages          | Experimental    | `oxlint-plugin-vize`, `@vizejs/vite-plugin-musea`, `@vizejs/musea-mcp-server`, and `@vizejs/wasm` may change APIs, commands, output, or workflow shape during alpha. |
| Incubating packages            | Incubating      | `@vizejs/fresco`, `@vizejs/fresco-native`, and editor extension packages are not part of the production-ready target yet.                                            |

## Required Gates

A production-ready claim for any supported surface requires current evidence for all of these:

- `vp run --workspace-root check:ci`
- `vp run --workspace-root coverage`
- `vp run --workspace-root coverage:source`
- `vp run --workspace-root coverage:source:branch`
- `vp run --filter './tests' test:check:fixtures`
- `node tools/npm/smoke-release-install.mjs --prepare-manifests --runtime-checks ...`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `vp exec pnpm audit --prod --audit-level moderate`
- `cargo audit --deny warnings`
- release-artifact signatures and SBOMs land on the GitHub Release (see
  [supply-chain.md](./supply-chain.md))
- package build coverage for the surface being promoted
- package install smoke coverage for the surface being promoted
- supported Node.js compatibility coverage
- native host smoke coverage when native binaries are involved
- real-world fixture coverage for the supported Vue/compiler/typecheck behavior
- Vue parity matrix coverage for compiler, type checking, Vite, and runtime smoke behavior
- release rollback instructions for npm, crates.io, GitHub Releases, docs, and editor channels

## Current Audit Snapshot

Local audit evidence from May 18, 2026:

- `vp run --workspace-root check:ci` passes.
- `cargo fmt --all -- --check` passes.
- `cargo clippy --workspace -- -D warnings` passes.
- `cargo test --workspace` passes.
- `vp exec pnpm audit --prod --audit-level moderate` reports no known vulnerabilities.
- `cargo audit --deny warnings` reports no RustSec vulnerabilities.
- `vp run --workspace-root coverage` passes the compiler fixture budget at `625/625`.
- `cargo llvm-cov --workspace --summary-only` reports Rust source coverage at
  `71.18%` lines, `76.39%` functions, and `71.92%` regions, above the `70%`
  release gate.
- `cargo +nightly llvm-cov -p vize_carton -p vize_armature -p vize_atelier_core --branch`
  reports core Rust branch coverage at `44.34%`, above the `40%` release gate.
- `node bench/test-inventory.mjs --json ...` reports `4,480` tracked cases across `607` files.

These results are strong alpha evidence, but they are not enough to call the whole repository
production ready. The formerly missing release-blocking checks are tracked by these issues and now
have explicit gates in CI and release docs:

- [#492](https://github.com/ubugeeei/vize/issues/492): source line and branch coverage gates, with function and region budgets included.
- [#493](https://github.com/ubugeeei/vize/issues/493): fresh-install smoke coverage for runtime package tarballs across supported OS and Node targets.
- [#494](https://github.com/ubugeeei/vize/issues/494): public Vue compiler and language-tools parity matrix.

The public compatibility baseline is documented in
[Vue Parity Matrix](./vue-parity-matrix.md).

## Exit Criteria For Removing Public Warnings

Do not remove the public work-in-progress or experimental warnings until:

- the promoted support scope is named in this document and in the stability guide
- the [support policy](./support-policy.md) covers the surface, including deprecation windows
- every required gate above is green on the release commit
- skipped production-relevant fixtures are either re-enabled or documented as known unsupported behavior
- dependency audits are blocking, or every temporary ignore has an owner and review date
- release notes include breaking changes, known limitations, and migration steps
- a maintainer has linked the verification evidence from the release checklist

## How To Answer The Readiness Question

When asked whether Vize is completely production ready, answer from the support scope:

- The whole repo is not completely production ready while preview, experimental, and incubating
  surfaces remain.
- The alpha-supported packages may be used for early production trials when the required gates pass
  and the release notes match the adopter's risk tolerance.
- Official Vue tooling remains the compatibility baseline whenever Vize output disagrees with it
  unless Vize documentation explicitly says otherwise.
