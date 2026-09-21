# Phase 6 — Task contracts, P6-7 through P6-13

> [!NOTE]
> Continuation of [phase-6-tasks.md](./phase-6-tasks.md) under the 350-line source budget. Same authority, same format; the TODO index in [phase-6.md](./phase-6.md) links each task to whichever file holds its contract.

## P6-7 — JS plugin SDK GA

**Start gate:** gated on P5-13 — GA hardens the cached plugin path.

**Lane:** F

**Deliverable:** the P4-16 spike hardened per charter #29: all four hook families (custom lint rules + autofix, custom fact providers, compile transform hooks, formatter/output hooks), batched napi visits, per-plugin cost attribution in lint output, content-keyed caching (P5-13), authoring docs and the `@vizejs/plugin-sdk` package; transform hooks locked to the single pre-canonical S2 point with determinism checks.

**Steps:**

- [ ] `npm/plugin-sdk/` and `crates/vize_vitrine/src/napi/plugin_sdk*`; the same input twice must produce the same output, enforced in CI
- [ ] Two real-world rules and one transform hook as fixtures

**Acceptance:** TS-51 GA — at least two real-world rules and one transform hook run deterministic, cached and cost-attributed; the parity bar unbroken (TS-11: a transform cannot exempt its output).

**Deps:** P5-13, P4-16.

**Non-goals:** ESLint compatibility (explicitly not a goal).

## P6-8 — Contract versioning and semver policy

**Landed 2026-09-22** — full record: [phase-6-records/p6-8.md](./phase-6-records/p6-8.md).

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** G

**Deliverable:** `davinci-road/contracts-compat-policy.md` (written before GA) and Marquette-style canonical serialization with additive/breaking classification for contract payloads, extending `crates/vize_marquette/src/compatibility*`; the conformance suite is versioned alongside the contracts.

**Steps:**

- [x] Classification for WIT world changes and payload schema changes in `crates/vize_marquette/src/contracts*`
- [x] Policy doc: what is additive, what is breaking, how versions move ([contracts-compat-policy.md](../contracts-compat-policy.md))

**Acceptance:** a deliberately breaking contract change is flagged by the classification tooling and a purely additive one is not (both as tests); the policy committed and linked from the WIT package.

**Deps:** P6-1a.

**Non-goals:** internal-format stability (charter #23 keeps it free until GA).

## P6-9 — External-consumer validation

**Start gate:** startable now — no open earlier-phase dependency (waits behind P6-2, P6-4b and P6-6).

**Lane:** H

**Deliverable:** each of the three contracts validated externally: the expression dialect by MoonBit (P6-4b), the output target by Volt (P6-6), and the input-dialect contract by a third-party guest built against the P6-2 SDK by someone other than the maintainer (a community Svelte or Astro prototype, for example).

**Steps:**

- [ ] `tests/external-consumers/` pins each external build against a tagged release
- [ ] Friction lists triaged

**Acceptance:** TS-50 covers all three builds without patching vize internals. **Review point:** the input-dialect guest must come from a third party; the record names who built it.

**Deps:** P6-2, P6-4b, P6-6.

**Non-goals:** in-tree non-Vue dialects (charter #1).

## P6-10 — Completion-metrics review

**Start gate:** gated on P5-14 — metrics are reviewed against the finished substrate.

**Lane:** I

**Deliverable:** `davinci-road/completion-metrics.md`: charter #35's pinned numbers against achieved — compile throughput, peak memory, keystroke p95, fact adoption, source-map coverage, matrices without orphans — with each miss explained or remediated.

**Steps:**

- [ ] Collect each number from its gate's recorded evidence, never re-estimated

**Acceptance:** the review committed; every metric either met with its evidence link or missed with its blocker.

**Deps:** P5-14.

**Non-goals:** moving any target after the fact.

## P6-11 — v1 go-no-go input package

**Start gate:** startable now — no open earlier-phase dependency (waits behind P6-10).

**Lane:** I

**Deliverable:** the go/no-go input assembled — parity matrices, budgets history, waiver/FP/FN ledgers, conformance results, corpus coverage — and wired into the checklist of `docs/release/v1-alpha-go-no-go.md` (charter #24).

**Steps:**

- [ ] Link each checklist item to its evidence

**Acceptance:** every checklist item links evidence or names its blocker. **Review point:** the maintainer accepts the package.

**Deps:** P6-10.

**Non-goals:** the go/no-go decision itself (the maintainer's).

## P6-12 — Communications decision

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** J

**Deliverable:** charter #45's decision point, brought forward by the Vue Fes Japan 2026 presentation (2026-10-24): what is published (architecture docs, a blog note, the talk material) and what stays internal, recorded in `davinci-road/communications.md`, with the documents that go public passing C-25's truth pass (they claim only what ships).

**Steps:**

- [ ] Draft the publication scope and the truth-pass checklist for each public document
- [ ] The maintainer records the decision; charter row #45 is amended by the maintainer in the same PR or a follow-up

**Acceptance:** decision recorded; every public-facing document lists its truth-pass evidence. **Review point:** the decision is the maintainer's alone.

**Deps:** none (phase-2 exit).

**Non-goals:** writing the talk itself.

## P6-13 — Phase exit

**Start gate:** gated on P5-14 — phase order.

**Lane:** X

**Deliverable:** the exit gate in [phase-6.md](./phase-6.md) evaluated inline — ticked only when satisfied, unticked lines naming their blocker, nothing softened — which closes Davinci's roadmap and hands the evidence to the v1 go/no-go.

**Steps:**

- [ ] Evaluate every gate line with evidence

**Acceptance:** every exit-gate line ticked with evidence or carrying a named blocker.

**Deps:** every other phase-6 task, P5-14.

**Non-goals:** v1 itself.
