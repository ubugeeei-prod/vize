# Vue Parity Matrix

This matrix defines which Vue compatibility surfaces are release-blocking for the v1 alpha line.
Official Vue tooling is the baseline whenever Vize output disagrees with it, unless a Vize guide
explicitly documents a narrower behavior.

## Baseline Versions

| Tool              | Baseline                                                                         | Gate                                            |
| ----------------- | -------------------------------------------------------------------------------- | ----------------------------------------------- |
| Vue runtime       | `vue@3.5.34` for published runtime smoke, `vue@3.6.0-beta.10` for fixture parity | `fresh-install-smoke`, `vue-parity`             |
| Vue compiler      | `@vue/compiler-sfc@3.6.0-beta.10`                                                | `vp run --filter './tests' test:check:fixtures` |
| Vue type checking | `vue-tsc@3.2.9` with `typescript@6.0.3`                                          | `vp run --filter './tests' test:check:fixtures` |
| Vite              | `vite@npm:@voidzero-dev/vite-plus-core@0.1.21`                                   | `fresh-install-smoke`, app e2e                  |
| Node.js           | `22` and `24`                                                                    | `node-engine-compat`, `fresh-install-smoke`     |

## Compatibility Surfaces

| Surface                                                 | Status                | Release-blocking evidence                                                                                    | Notes                                                                                                                                           |
| ------------------------------------------------------- | --------------------- | ------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| SFC parse and `<script setup>` compile                  | Alpha-supported       | `@vue/compiler-sfc` accepts parity fixtures in `test:check:fixtures`; compiler fixture coverage is `625/625` | Vize must not reject fixtures accepted by the official compiler baseline.                                                                       |
| Template compilation, directives, slots, and asset URLs | Alpha-supported       | compiler fixture coverage, Vite plugin runtime smoke, app e2e                                                | Official Vue output remains the tie-breaker for unlisted edge cases.                                                                            |
| `vize check` diagnostic file surface                    | Alpha-supported       | `vue-tsc` parity in `tests/snapshots/check/toolchain-parity.ts`                                              | For files with errors, Vize and `vue-tsc` must agree on the Vue files that own diagnostics and share TypeScript diagnostic codes.               |
| `@vizejs/native` NAPI runtime                           | Alpha-supported       | `fresh-install-smoke` requires `compileSfc` from installed tarballs on macOS, Linux, and Windows             | Root package and compatible platform package must be installed from tarballs together.                                                          |
| `@vizejs/vite-plugin` production build                  | Alpha-supported       | `fresh-install-smoke` runs `vite build` from installed tarballs on Node 22 and 24                            | Covers a real Vue app shell instead of only verifying package entrypoints.                                                                      |
| Vapor compiler mode                                     | Experimental          | VDOM/Vapor fixture coverage                                                                                  | Experimental: API, output, and workflow shape may change with Vue Vapor upstream changes before v1 stable.                                      |
| Vapor mode Options API                                  | Experimental          | Rust unit tests and Vapor fixture coverage                                                                   | Experimental opt-in: Options-API-authored components compiled for Vapor mode.                                                                   |
| Standalone CDN build                                    | Experimental          | release package smoke                                                                                        | Experimental opt-in: single-file standalone bundle usable directly from a CDN with no bundler step.                                             |
| Zero-JavaScript prerenderer (Island)                    | Planned               | n/a — not yet implemented                                                                                    | Roadmap surface targeting production-ready once shipped: Island-architecture prerendering that emits zero client JavaScript for static islands. |
| SSR-specific compiler and lint behavior                 | Preview               | Rust unit tests and fixture coverage                                                                         | Production SSR apps must validate their own hydration and renderer constraints.                                                                 |
| Nuxt, Rspack, unplugin, and Musea integrations          | Compatibility preview | package build/check jobs and release smoke installs                                                          | Host-framework compatibility must be validated before a production rollout.                                                                     |
| Editor language features                                | Incubating            | extension packaging and LSP tests                                                                            | Vize editor support should run alongside official Vue language tooling during alpha.                                                            |
| WASM package runtime                                    | Experimental          | release package smoke                                                                                        | API and runtime shape may change during alpha.                                                                                                  |

## Required Release Gates

The release commit must pass these gates before any alpha-supported Vue surface can be described as
production-trial ready:

- `vp run --workspace-root coverage`
- `vp run --workspace-root coverage:source`
- `vp run --workspace-root coverage:source:branch` for nightly branch coverage on core Rust
  compiler crates
- `vp run --filter './tests' test:check:fixtures`
- `node tools/npm/smoke-release-install.mjs --prepare-manifests --runtime-checks ...`
- `node --test tests/tooling/release-readiness.test.ts tests/tooling/github-workflows.test.ts`

Preview and experimental surfaces may ship only with release notes that call out their status and
known unsupported behavior.
