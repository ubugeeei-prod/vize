# Phase 6 — Task contracts, P6-1a through P6-6

> [!NOTE]
> Full contracts for [Phase 6 — Extension Contracts GA](./phase-6.md), early re-cut 2026-09-21; [phase-6-tasks-later.md](./phase-6-tasks-later.md) continues them under the 350-line source budget. Charter #15's two tiers govern: first-party dialects stay compiled in behind traits and features, external ones cross the serialized WIT contract with coarse-grained interfaces only; charter #39 governs new runtime dependencies (pin, audit, feature isolation).

## P6-1a — Input-dialect WIT world and capability handshake

**Landed 2026-09-22** — full record: [phase-6-records/p6-1a.md](./phase-6-records/p6-1a.md).

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** A

**Deliverable:** `crates/vize_extension_sdk/wit/input-dialect.wit`: block in → S1 surface tree and S2 page out, as coarse-grained calls (one call per block, canonical-ABI copy cost paid once), plus the `get-capability` handshake — integer protocol version and feature strings (the Swift import) — shared by every world.

**Steps:**

- [x] WIT package `vize:contracts@0.1.0` with the handshake interface and the input world; serialized payloads are the S1/S2 folio `Full` forms (schema-versioned per P2-17; the S1 page is new, [folio-format-s1.md](./folio-format-s1.md))
- [x] A host-side golden exchange: a guest stub echoing a committed S1/S2 payload
- [x] Register the TS-48 command in [test-suites.md](./test-suites.md): `cargo test -p vize_extension_host --features extension-host --test wit_golden` (wasmtime stays behind the feature, charter #39)

**Acceptance:** TS-48 for the input world: capability negotiation including rejection of a mismatched version with its exact error, and byte-equal serialized payloads in the golden exchange.

**Deps:** none (phase-2 exit).

**Non-goals:** the expression and output worlds (P6-1b, P6-1c); in-process hosting (P6-3).

## P6-1b — Expression-dialect WIT world

**Landed 2026-09-22** — full record: [phase-6-records/p6-1b.md](./phase-6-records/p6-1b.md).

**Start gate:** gated on P4-5a — the world exports projection mapping rows in the unified model.

**Lane:** A

**Deliverable:** `crates/vize_extension_sdk/wit/expression-dialect.wit`: environment + expression body in → analysis facts (referenced bindings, const-ness, spans — P2-5b's capability contract) and a checkable projection with span links out (charter #14).

**Steps:**

- [x] World definition; fact payloads as P4-2 α pages, projection rows as P4-5a `ProjectionMapping`
- [x] Golden exchange with a stub expression guest

**Acceptance:** TS-48 for the expression world: negotiation and byte-equal payloads.

**Deps:** P6-1a, P4-1a, P4-5a.

**Non-goals:** MoonBit itself (P6-4b).

## P6-1c — Output-target WIT world

**Start gate:** gated on P3-9 — the world carries the S4 structured emission document.

**Lane:** A

**Deliverable:** `crates/vize_extension_sdk/wit/output-target.wit`: canonical S3/S2 in → emitted document (P3-9's span-carrying S4 document) out.

**Steps:**

- [x] World definition; golden exchange with a stub target emitting a fixed document

**Acceptance:** TS-48 for the output world: negotiation and byte-equal payloads.

**Deps:** P6-1a, P3-9.

**Non-goals:** Volt itself (P6-6).

**Landed 2026-09-23** — full record: [phase-6-records/p6-1c.md](./phase-6-records/p6-1c.md).

## P6-2 — Prebuilt versioned SDK

**Landed 2026-09-22** — full record: [phase-6-records/p6-2.md](./phase-6-records/p6-2.md).

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** B

**Deliverable:** versioned, prebuilt contract artifacts published per release — WIT bindings, a Rust SDK crate `crates/vize_extension_sdk/`, JS/TS types in `npm/extension-sdk/` — so no consumer ever compiles vize internals (the Swift macro-crisis countermeasure).

**Steps:**

- [x] SDK crate with no dependency on any vize implementation crate (a tooling test asserts its dependency set)
- [x] A hello-world input dialect built against the SDK tarball alone in CI

**Acceptance:** the hello-world dialect builds from the packed SDK with no path dependency into the workspace; the dependency-set test proven to fail on an injected implementation edge.

**Deps:** P6-1a.

**Non-goals:** publishing automation beyond the existing release pipeline.

## P6-3 — In-process wasmtime hosting lane

**Landed 2026-09-22** — full record: [phase-6-records/p6-3.md](./phase-6-records/p6-3.md).

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** C

**Deliverable:** `crates/vize_extension_host/`: feature-gated (`extension-host`) in-process hosting of contract guests under wasmtime, sharing the WIT contract with the out-of-process transport, with per-guest fuel and memory limits. wasmtime is admitted under charter #39 (pinned, audited, only behind this feature).

**Steps:**

- [x] Host crate; the same guest binary runs out-of-process and in-process
- [x] A tooling test: nothing outside `extension-host` features depends on `wasmtime` ([`davinci-extension-host-deps.test.ts`](../../../tests/tooling/davinci-extension-host-deps.test.ts))

**Acceptance:** the same guest passes TS-48 in both hosting modes; a guest exceeding its fuel budget is stopped with the exact error; the dependency test green and proven to fail on an injected edge; `cargo audit --deny warnings` green.

**Deps:** P6-1a.

**Non-goals:** sandboxing JS plugins (P6-7).

## P6-4a — MoonBit hosting spike

**Landed 2026-09-22** — decided: the pinned native `moonc` as a child process behind the `MooncHost` boundary (no wasm build of `moonc` exists; the `moonc-worker` Node build is the fallback); the spike is kept with tests as `crates/vize_dialect_moonbit/` — full record: [phase-6-records/p6-4a.md](./phase-6-records/p6-4a.md). Review point open.

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** D

**Deliverable:** the hosting decision for `moonc`, measured: whether a vendored, pinned wasm build of `moonc` with a virtual filesystem runs under wasmtime (artifact, imports, wasm-gc support), or falls back to a Node sidecar behind the same capability boundary. MoonBit's documented launchers are Node-based, which is why this is a spike first.

**Steps:**

- [x] Try the wasm artifact under wasmtime; record the imports and failures exactly
- [x] Record the decision and its measurements in the task record

**Acceptance:** decision recorded with reproduction commands; the spike code kept with tests or deleted, and the PR says which. **Review point:** the maintainer accepts the hosting choice.

**Deps:** none (phase-2 exit).

**Non-goals:** the dialect (P6-4b).

## P6-4b — MoonBit expression dialect

**Start gate:** gated on P4-5b — the dialect's projection is an instance of the single projection.

**Lane:** D

**Deliverable:** charter #28's MoonBit dialect: a generated `.mbti` binding environment from S2 scope facts (props, refs, composables as MoonBit signatures), template expressions projected to `.mbt` bodies, `moonc build-package` check-only, diagnostics span-mapped back, and the `moonc` version inside the fact cache key.

**Steps:**

- [ ] `crates/vize_dialect_moonbit/` over the P6-1b world and the P6-4a hosting choice
- [ ] Matrix fixtures with MoonBit expressions compiled and checked end to end
- [ ] Register the TS-49 command in [test-suites.md](./test-suites.md)

**Acceptance:** TS-49 — span-mapped diagnostics exact over the `.mbti`/`.mbt` fixtures, with the `moonc` version in the cache key.

**Deps:** P6-4a, P6-1b, P4-5b.

**Non-goals:** MoonBit outside template expressions.

## P6-5 — ExprRef validation report

**Start gate:** startable now — no open earlier-phase dependency (waits behind P6-4b).

**Lane:** D

**Deliverable:** `docs/davinci/plan/exprref-validation.md`: what the second expression implementation revealed about `ExprRef` — capability-set gaps, span-model fit, projection-contract adequacy — with the fixes merged or explicitly deferred with rationale (charter #28's accepted late-validation risk gets its bill here).

**Steps:**

- [ ] Write the report from P6-4b's findings; land or defer each fix

**Acceptance:** the report committed; every finding marked `fixed` (with its PR) or `deferred` (with its rationale).

**Deps:** P6-4b.

**Non-goals:** a third dialect.

## P6-6 — Volt output-target exercise

**Mechanical run 2026-09-23** — `examples/volt-target` emits one HEEx module
through the output world; the recorded command exits 0. **Review point
open:** the Volt maintainer has not signed off, so the phase index stays
open. Record: [phase-6-records/p6-6.md](./phase-6-records/p6-6.md).

**Start gate:** startable now — no open earlier-phase dependency (waits behind P6-1c).

**Lane:** E

**Deliverable:** with the Volt (Elixir) maintainer, an output-target guest emitting for the Elixir host through the P6-1c world, run end to end in `examples/volt-target/`; findings feed the contract before the GA freeze.

**Steps:**

- [x] Guest and host harness; a documented end-to-end run
- [x] Contract-change list, each item resolved or deferred

**Acceptance:** the end-to-end run reproduces from its recorded command; the change list resolved. **Review point:** requires the Volt maintainer's participation and sign-off.

**Deps:** P6-1c, P6-3.

**Non-goals:** maintaining Volt.
