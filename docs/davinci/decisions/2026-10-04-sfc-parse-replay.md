# Historical isolated SFC parse replay

This is the bounded follow-up for [P1 #6189](https://github.com/ubugeeei-prod/vize/issues/6189).
It does not close the issue, change production, or measure the type checker.

## Evidence and unresolved questions

The original [Criterion run 35204770455](https://github.com/ubugeeei-prod/vize/actions/runs/35204770455)
used base `1649cd0a40ec22aa887f15b1509a27de007ccf88` and head
`947879a5427334b6f2524c92a5cd37a19fdc0b3d` on the Linux reference runner.
Its complete job log records complex at 09:26:58 UTC for base,
`[4.5420, 4.5872, 4.6306] us`, and 09:42:54 UTC for head,
`[5.3512, 5.3588, 5.3689] us`. All base suites ran before the head suites;
other suite builds and measurements intervened for about 16 minutes.
That ordering is a confound, not evidence that the slowdown was noise.
The artifact-list API currently returns no artifacts for that run; its
complete job log remains available. The log intervals cannot recover raw samples.

The issue already retains six local alternating pairs on Rust 1.98.0,
with median head/base ratios 1.0731 for medium and 1.0691 for complex.
Those observations neither reproduce the original Linux magnitude nor
identify a runtime or code-layout cause. A later compiler control was noisy.

Both historical revisions have identical `sfc_parse.rs` bytes, SHA256
`eda70eb21f390c6d8fe72a354534db97fbac2f1e3f906ee4e5295a11c328e1e8`,
and unchanged parser source. Normal dependency edges changed, so equal
source or symbol extents do not establish equal generated instructions.
The original setup banner reports stable 1.98.1, while both source pins are
1.98.0; the original Cargo command did not record its actual compiler.
The replay explicitly selects 1.98.0 for both builds and retains `rustc -vV`
plus verbose Cargo commands. It does not claim to resolve the original compiler.

## Decision and executable protocol

Use the registered manual `Criterion Bench` workflow's explicit
`historical_sfc_parse=true` route. The workflow checkout supplies the trusted
driver; base/head remain the two immutable historical commits. Reject any
other pair, mixed Vapor/production route, changed frozen benchmark or locked
dependencies, changed release settings, or external compiler/profile overrides.
Validate exact checkout identities and base ancestry. Normal benchmark routes
retain their existing dispatch-head, ancestry and budget gates.

Each of three reference runner instances builds both existing benchmark
executables before measurements, with separate fresh target directories,
`cargo +1.98.0 bench --locked --no-run`, the source release settings
(fat LTO, one codegen unit), and Wild 0.9.0. Runner 2 reverses build order.
Do not insert builds or symbol inspections into the measured pair sequence.

Measure only the unchanged simple, medium and complex timed bodies.
Each case gets six adjacent base/head pairs, alternating AB/BA three times
each. Case order is also balanced across the six pairs. Direct execution
passes `--bench --exact sfc_parse/<case>` and the original Criterion 0.8.2
defaults explicitly: 100 samples, 3 seconds warmup, 5 seconds measurement,
100000 resamples and 95% confidence. Plot generation is disabled.
Each side/case/pair has its own Criterion home and complete saved baseline.
The throughput control and whole-CLI benchmarks are outside this experiment.

Retain every iteration/time sample, Criterion estimate and metadata, command
stdout/stderr, execution chronology, source/lock/config files, actual compiler,
runner/CPU metadata, binary hashes, symbol addresses/extents and disassembly.
Always upload partial evidence on failure; retain artifacts for 30 days.
Keep linked experiment receipts in the issue and this record before expiry.

For each runner/case, report all six head/base median-estimate ratios, their
median and observed range, and separate base-first/head-first medians.
A fixed-seed 10000-resample percentile interval resamples six whole pairs;
it is conditional on that runner and its built binaries, excluding build/layout
and between-runner uncertainty. Do not pool Criterion iterations as independent
A/B runs, discard inconvenient observations, or call the range a confidence interval.

## Remaining work

Run the exact source lane and inspect all three runner packets and order effects.
If a smaller signal persists, inspect emitted parser code and layout before
choosing a bounded control or real fix. If results differ by runner/order,
report those differences and uncertainty. Original compiler identity, cause,
user-visible impact and issue closure remain unresolved. A green workflow
means this recipe completed; it does not certify performance or a 10x target.
