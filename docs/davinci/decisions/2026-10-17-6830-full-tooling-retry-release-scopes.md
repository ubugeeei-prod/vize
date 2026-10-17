# Preserve retry and release input contracts after workflow changes

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Full Check [38988499789](https://github.com/ubugeeei-prod/vize/actions/runs/38988499789)
exposed two stale tooling contracts. Neither failure requires changing the
production workflow or reducing its checks.

The Rust retry test read the retired download `name` field. The current
consumer selects a verified immutable artifact ID. Exercise that real selector
with the producer's source-bound upload name: a failed-only rerun keeps the
completed producer's ID, while a full rerun receives a replacement ID.
Foreign run/source selection, changed archive bytes and stale source receipts
remain rejected. The existing archive-download tests cover bounded transfer
retry, cleanup, digest enforcement and fatal completion gates.

The one-time v0.457.0 consumer contract was initially restored to the explicit
broad inventory because its ledger and Git-selected authority were runtime
inputs. That unexecuted duplicate was subsequently
[retired](./2026-10-17-6830-published-consumer-supplement.md) when main gained
the canonical recovery path. Its stale inventory registration was removed too.
The two existing contracts now importing the canonical runtime native probe
retain the planner's broad fallback; all other audited import closures remain
complete. Exact inventory equality and selection for unknown runtime inputs
remain checked, without changing production input selection.

Main #9317 subsequently added `legacy-check-qualification.test.ts`. Its actual
driver harness reads Rust modules and embeds their paths and bytes at runtime,
so it retains the existing broad input fallback. Full Check
[38993084178](https://github.com/ubugeeei-prod/vize/actions/runs/38993084178)
caught the missing inventory entry; register that existing contract without
changing its 24 scenarios, the selector, or the audited narrow import closures.
