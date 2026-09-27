# Keep native declarations in PR checks and UI acceptance in merge queue

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) and
[#6864](https://github.com/ubugeeei-prod/vize/issues/6864).

## Actual commands and inputs

Fresco `check:generated` invokes `vp exec napi build --platform --profile ci`
against `crates/vize_fresco/Cargo.toml`, with package `vize_fresco` and feature
`napi`, into a temporary directory. It compares generated `index.d.ts` bytes
and parses generated and committed loader exports. It compiles Rust for the
declaration contract; it does not load an addon or execute terminal or UI
behavior.

Fresco `check:types` stages the published manifest, loader and declarations,
`tests/types/consumer.ts`, and `tsconfig.types.json` in an isolated consumer.
It invokes `vp exec tsc --noEmit -p <consumer>/tsconfig.types.json`, using the
TypeScript catalog dependency. This is a static compiler-backed type check,
not a byte-only check. It neither emits nor executes the consumer, and it
does not discover tsgo or require a native runtime.

UI `check` runs SFC lint and renderer checks, static checks, the story-testbed
check, and Vapor runtime acceptance in that order. SFC lint discovers the
workspace Corsa executable; renderer checks use the native compiler; Vapor
acceptance executes native-generated SSR, mount and hydration outputs and
enforces the known-failure ledger.

## Decision and verification

- Keep both Fresco checks in PR and merge-group JS jobs, in their original
  order. Keep the full 16-package ordinary JS test group.
- Run `npm/native build:ci` and the full `npm/ui check` only for merge-group
  events. Preserve the complete merge sequence: JS package tests, Fresco
  generation, consumer types, native CI build, then UI checks.
- Keep the existing JS job in the strict source-result gate; a failing
  declaration, type check or merge acceptance step fails that job.

Scoped workflow tests execute selected step commands with a recording CLI
and check the PR, merge-group and no-JS cases. They also run the actual Fresco
scripts with a generated-contract fixture and a real TypeScript compiler,
without a native runtime or a host Rust build. Actions must confirm the PR
skips the UI acceptance step and the merge queue still runs the full tail.
