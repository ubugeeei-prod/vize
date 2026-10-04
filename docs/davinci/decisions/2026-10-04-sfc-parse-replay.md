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
Copy both built ELF files into the uploaded packet before timing, verify their
hashes against the originals, and record retained relative paths. Execute the
original build paths so archival custody does not change the measured argv/path.
Always upload partial evidence on failure; retain artifacts for 30 days.
Keep linked experiment receipts in the issue and this record before expiry.

For each runner/case, report all six head/base median-estimate ratios, their
median and observed range, and separate base-first/head-first medians.
A fixed-seed 10000-resample percentile interval resamples six whole pairs;
it is conditional on that runner and its built binaries, excluding build/layout
and between-runner uncertainty. Do not pool Criterion iterations as independent
A/B runs, discard inconvenient observations, or call the range a confidence interval.

## Initial completed hosted experiment

Exact harness source `4810bf6f14597e9ee7e50e20d969e2fe0395415c`,
[replay 37183831294](https://github.com/ubugeeei-prod/vize/actions/runs/37183831294),
succeeded on all three reference instances. Jobs are `111381562127`,
`111381562224` and `111381562292`; artifacts are `11296038897`,
`11296336813` and `11296560828`, respectively. Each packet was independently
checked against frozen source/lock hashes, two completed pre-window builds,
36 unique chronologically ordered windows, and all 3600 raw sample pairs.
The 10800 Criterion sample pairs are retained observations, not independent A/B runs.

| Runner | Case    | Paired median head/base | Conditional 95% interval | Base first | Head first |
| ------ | ------- | ----------------------: | ------------------------ | ---------: | ---------: |
| 1      | simple  |                  0.9739 | 0.9431–0.9903            |     0.9455 |     0.9776 |
| 1      | medium  |                  1.0366 | 0.9936–1.0752            |     1.0391 |     1.0341 |
| 1      | complex |                  1.0543 | 0.9958–1.0797            |     0.9961 |     1.0661 |
| 2      | simple  |                  0.9613 | 0.9222–0.9970            |     0.9887 |     0.9282 |
| 2      | medium  |                  1.0377 | 1.0038–1.0657            |     1.0439 |     1.0083 |
| 2      | complex |                  1.0933 | 1.0162–1.1124            |     1.1123 |     1.0744 |
| 3      | simple  |                  0.9485 | 0.9245–0.9939            |     0.9472 |     0.9498 |
| 3      | medium  |                  1.0410 | 1.0038–1.0784            |     1.0056 |     1.0467 |
| 3      | complex |                  1.0945 | 1.0079–1.1318            |     1.1194 |     1.0696 |

Medium paired medians are +3.66% to +4.10%; complex is +5.43% to +9.45%.
Runner 1's medium and complex conditional intervals include 1; runners 2/3
exclude 1. Simple is faster at every runner median. Order-specific estimates
and runner differences remain visible; these results do not reproduce the
original 18% magnitude or establish a cause or absence of regression.

Actual Rust is `1.98.0 (88d9e12ae 2026-08-18)` on AMD EPYC. All three
packets report identical base SHA256
`66d7c178aed647eb7d5af69f2919ee61973b7627c487f209f9a8c9387b16f72b`
and head SHA256
`e1e8cb8c46161fbe9f7bd0a63a06e7eab69203783c045ad0dfced1b925b0fdba`.
The parser symbol moves `0x319ba0` to `0x319d00` (+352 bytes), with equal
`0x3a41` extent. That is layout evidence, not proof of identical instructions
or of layout causing the timing difference. These initial successful packets
lack the actual ELF files; preserve them as historical and require a fresh
packet after the independent review's binary-custody repair.

## Remaining work

Initial source `1bc535ddd748d196733e3d5b4bf38fe9314d45cc` started replay
`37183563214` and full Check `37183565314`. The online security audit rejected
the copied generated stable installer pin as unreachable. The replay wrapper
now uses verified upstream master `7e38f4b43b4db5c8dd498af069a4f6196df1d067`
with the same explicit 1.98.0 input. The initial attempts were superseded;
partial artifacts and failed audit remain historical, with no performance credit.
The initial replay itself stopped after its first successful locked build:
Cargo `-vv` interleaved labelled build-script stdout with JSON messages.
Artifact selection now parses only Cargo JSON lines, requires successful
build completion and one unambiguous bench executable, and preserves the
entire stdout/stderr streams. Regression guards cover the actual mixed stream.
Initial partial artifact IDs are `11295943362` (runner 1), `11295709076`
(runner 2) and `11296525246` (runner 3); none contains timed observations.
Independent peer review identified that hashes and disassembly alone did not
retain the original ELF bytes. The packet now retains verified copies of both
measured binaries, enabling later independent section/layout and hash inspection.

Run the exact source lane and inspect all three runner packets and order effects.
If a smaller signal persists, inspect emitted parser code and layout before
choosing a bounded control or real fix. If results differ by runner/order,
report those differences and uncertainty. Original compiler identity, cause,
user-visible impact and issue closure remain unresolved. A green workflow
means this recipe completed; it does not certify performance or a 10x target.
