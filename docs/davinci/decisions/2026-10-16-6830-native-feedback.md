# Native product feedback and historical performance qualification

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The native phase workflow mixes product regression tests with optimized-build
benchmarking, raw phase captures and reconstruction of historical generated
code. Its last three successful PR runs took
[18m29s](https://github.com/ubugeeei-prod/vize/actions/runs/38608723570),
[16m02s](https://github.com/ubugeeei-prod/vize/actions/runs/38608539016) and
[15m45s](https://github.com/ubugeeei-prod/vize/actions/runs/38608524933).
Repeated compilation in the `ci-opt` profile and serial CLI/editor/reference-path
qualification dominate. A recent seven-minute failure is an incomplete run,
not evidence that the complete campaign became faster.

Move the complete phase and historical receipt campaign to scheduled/manual
qualification. Ordinary PRs and merge candidates retain affected product tests,
using the existing Cargo dependency graph and compatible test caches. They do
not rebuild a second optimized workspace solely to produce benchmark receipts.

Most CLI, Canon and Maestro assertions already run in the affected Rust suite.
The merge queue uses the full nextest profile with the real TypeScript native
runtime required. Three additional product families must remain explicit:

- The ignored Canon snapshot-source native law, with its pinned native runtime
  and source capture.
- The Canon projection differential suite with `legacy-differential` enabled.
- The eight actual source-built LSP Node test files for computed inlays, editor
  configuration, private imports and project navigation. Use one candidate CLI
  and one shared pinned Vue provider; retain all existing assertions and
  required-runtime flags.

Run these affected families in parallel with the other source checks. Their
failures and unexpected skips must fail the source aggregate. Current complete
TS-40 projection goldens remain product assertions; no snapshot is rebased by
this CI change.

## Historical receipt debt

Baseline run [38805236632](https://github.com/ubugeeei-prod/vize/actions/runs/38805236632)
and this PR both passed the current projection matrix, then failed the old
template-emission reconstruction. The retained raw Canon text matches the
current committed golden exactly, including its 37,846-byte length; the original
Vue input hash also matches. Intentional generic-library and listener inference
changes added 2,029 bytes that the historical reconstruction does not remove.
That explains the old comparison of 36,507 reconstructed bytes to 34,478 bytes.

The failing historical assertion remains visible in scheduled/manual reports.
Do not rewrite its frozen baseline, silently accept a different digest, or
claim that this historical proof passed. Its future repair or retirement is
qualification debt tracked by #6830. Extending a generated-source rewind after
every valid product change is not a daily development prerequisite.

This separates semantic regressions from historical benchmark qualification;
it does not make a benchmark artifact from one source valid for another source.
The scheduled campaign retains its source custody, raw captures and explicit
failure reporting.
