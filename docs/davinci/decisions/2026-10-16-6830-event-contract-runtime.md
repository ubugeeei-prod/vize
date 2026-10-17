# Run the event contract against the targeted Vapor runtime

**Issue:** [#6830](https://github.com/ubugeeei-prod/vize/issues/6830)

## Cause

The scheduled [Davinci Lean run](https://github.com/ubugeeei-prod/vize/actions/runs/38882290854/job/116911649438)
on `cd861aa001f7ea458f8b0147e5b9c850b9db1c87` passed the Lean build,
proof audit, reference differentials and preceding Rust runtime checks. Its
last Chromium test failed on eight candidate observations: `vapor/prefix`
and `vapor/plain` for `temporal-dead-zone`, `missing-reference`,
`null-reference` and `value-reference`.

These fixtures intentionally throw `ReferenceError` or `TypeError`. Their
contract requires Vue's component error handler to receive each error.
Instead, the browser reported unhandled page errors.

[#8986](https://github.com/ubugeeei-prod/vize/pull/8986) aligned generated
listeners with the supported release-candidate runtime: `on` wraps its
handler in `createInvoker`. The event harness still mounted candidate output
with Vue `3.6.0-beta.10`, whose `on` calls `addEventListener` directly.
The generated listener therefore bypassed Vue's error handler only in this
outdated execution environment.

## Decision

Candidate Vapor engines use the existing `vue-vapor-release.mjs` provider,
which resolves the `vue-vapor-runtime` alias pinned by `npm/ui/package.json`
and authenticates the matching official compiler version. This is the same
`3.6.0-rc.9` runtime used by the other mounted and upstream parity harnesses.
The event report includes its version.

The stable and beta official reference engines retain their existing
runtimes and observations. Candidate VDOM, native JavaScript references,
Rust-owned inputs, expected traces, five handler mutations and rejection of
every unhandled page error are unchanged. No compiler behavior or Lean
validation is removed or changed. The remaining Lean reference package stays
until its laws and differentials are ported, as recorded in the
[Kani migration](2026-10-13-8840-html-content-model-kani.md#remaining-lean).

## Validation

The eight exact emitted artifacts from the failed hosted log were replayed
without editing their code. All eight reproduce the failure on beta.10; all
eight route both click errors through Vue on rc.9 and rc.11. Both matching
official release-candidate compilers also pass all four original scenarios.
These 32 Chromium observations establish the runtime mismatch independently
of a regenerated compiler output or changed diagnostic expectation.

The existing five workflow contract tests pass, including rejection of a
missing `--ignored` argument and failure-ignore settings. Scoped lint and
formatting pass. The complete existing ignored Chromium test passes locally:
39 scenarios, 314 mounted observations, five detected mutations, five native
JavaScript oracles and all 15 historical reference differences. The Rust test
finishes in 53.21 seconds. Full hosted acceptance requires a fresh exact-head
Davinci Lean run; the earlier scheduled run remains recorded as failed.
