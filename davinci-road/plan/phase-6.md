# Phase 6 — Extension Contracts GA (provisional decomposition)

> [!WARNING]
> Provisional; re-cut at phase-5 exit. Suites referenced as TS-n from
> [test-suites.md](./test-suites.md).

## TODO index

- [ ] P6-1 WIT worlds + capability handshake + compat policy
- [ ] P6-2 Prebuilt versioned extension SDK
- [ ] P6-3 In-process wasmtime hosting lane
- [ ] P6-4 MoonBit expression dialect
- [ ] P6-5 `ExprRef` validation report
- [ ] P6-6 Volt non-JS host exercise
- [ ] P6-7 JS plugin SDK GA
- [ ] P6-8 Contract versioning + semver policy
- [ ] P6-9 External-consumer validation
- [ ] P6-10 Completion-metrics review
- [ ] P6-11 v1 go/no-go input package
- [ ] P6-12 Communications revisit
- [ ] P6-13 Phase exit

---

**P6-1 WIT worlds.** `contracts/wit/` defining three worlds (input dialect:
block in → S1/S2 surface tree out; expression dialect: env + body in →
analysis facts + projection out; output target: canonical S3/S2 in → emitted
document out) — coarse-grained interfaces only (canonical-ABI copy cost);
`get-capability` handshake with integer protocol version + feature strings
(Swift import); `davinci-road/contracts-compat-policy.md` written before GA.
_Accept:_ TS-48 golden exchanges; policy reviewed.

**P6-2 Prebuilt SDK.** Versioned, prebuilt artifacts of the contract types
(WIT bindings + Rust SDK crate + JS/TS types) published per release — no
consumer ever compiles vize internals (the Swift macro crisis
countermeasure). _Accept:_ a hello-world dialect builds against the SDK
tarball alone.

**P6-3 wasmtime lane.** Feature-gated (`extension-host`, charter #39)
in-process hosting of contract guests under wasmtime, sharing the WIT
contract with out-of-process transport; resource limits (fuel/memory) per
guest. _Accept:_ same guest binary passes TS-48 in both hosting modes.

**P6-4 MoonBit dialect.** Vendored pinned wasm build of `moonc` with a
virtual FS — **spike first**: MoonBit's documented launchers are Node-based
and require a wasm-gc-capable runtime, so verify the artifact/imports/wasm-gc
under wasmtime, else fall back to a Node sidecar behind the same capability
boundary. Generated `.mbti` binding environment from S2 scope facts
(props/refs/composables as MoonBit signatures); template expressions
projected to `.mbt` bodies; `moonc build-package` check-only; diagnostics
span-mapped back; moonc version inside the fact cache key. _Accept:_ TS-49;
matrix fixtures for MoonBit expressions compile/check end-to-end; hosting
decision documented.

**P6-5 `ExprRef` validation report.** What the second expression
implementation revealed about the abstraction (capability set gaps, span
model fit, projection contract adequacy) — budgeted fix time included
(charter #28's accepted late-validation risk gets its bill here). _Accept:_
report committed; abstraction fixes merged or explicitly deferred with
rationale.

**P6-6 Volt exercise.** With the Volt (Elixir) maintainer: an output-target
guest emitting for the Elixir host through the WIT contract; findings feed
the contract before GA freeze. _Accept:_ documented end-to-end run;
contract-change list resolved.

**P6-7 JS plugin GA.** The P4-16 spike hardened per charter #29: four hook
families, batched napi visits, per-plugin cost attribution in lint output,
content-keyed caching (P5-13); authoring docs + `@vizejs/plugin-sdk` package;
transform hooks locked to the pre-canonical S2 point with determinism checks
(same input twice ⇒ same output, CI-enforced). _Accept:_ TS-51 with ≥2
real-world rules and 1 transform hook; parity bar unbroken (TS-11).

**P6-8 Versioning.** Marquette-style canonical serialization + additive/
breaking classification for contract payloads; semver policy doc; contract
conformance suite versioned alongside. _Accept:_ a deliberately-breaking
change is flagged by the classification tooling.

**P6-9 External validation.** **Each of the three contracts** is validated
externally: expression dialect by MoonBit (P6-4), output target by Volt
(P6-6), and the **input-dialect contract by a third-party guest** (e.g. a
community Svelte/Astro prototype built against the P6-2 SDK by someone other
than the maintainer). _Accept:_ TS-50 covers all three; friction lists
triaged.

**P6-10 Metrics review.** Charter #35's pinned numbers vs achieved (compile
throughput, peak memory, keystroke p95, fact adoption, source-map coverage);
misses explained or remediated. _Accept:_ review doc committed.

**P6-11 v1 package.** Go/no-go input assembled: parity matrices, budgets
history, waiver/FP/FN ledgers, conformance results, corpus coverage — wired
into `docs/release/v1-alpha-go-no-go.md`'s checklist (charter #24).
_Accept:_ review point — maintainer accepts the package.

**P6-12 Communications revisit.** Charter #45 decision point: publish the
architecture docs / blog note or stay internal. _Accept:_ decision recorded.

**P6-13 Phase exit.**

- [ ] TS-48..51 green; contracts documented with semver policy
- [ ] MoonBit + Volt + JS-rule validations complete; `ExprRef` report closed
- [ ] Completion metrics reconciled; v1 package delivered
