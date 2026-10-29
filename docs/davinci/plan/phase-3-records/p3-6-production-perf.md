# P3-6 production comparison protocol

`Criterion Bench` has a bounded `production_only=true` mode. Three independent
Blacksmith 32 vCPU Ubuntu 24.04 jobs compile the same exact head, then measure
the committed fixture SFC corpus through the four shipping adapter shapes from
P3-17: DOM inline, DOM separate module, SSR, and Vapor inline. The option builder
mirrors `davinci_production_reach/shapes.rs`, including script and style ids,
scoped-style inference, default DOM compiler options, and source-level Vapor
overrides. The SSR shape retains its SSR fallback for explicit Vapor sources.

```sh
gh workflow run criterion-bench.yml --ref <head-branch> \
  -f base_sha=<40-character-ancestor-sha> \
  -f head_sha=<40-character-head-sha> \
  -f production_only=true
```

This mode measures selected and forced retained compiles **on the same head**;
`base_sha` establishes provenance and ancestry, not a second timed revision.
The existing template Criterion and dialect comparison modes are unchanged.
The SFC feature `davinci-production-bench` enables only scoped backend selectors;
it does not enable the retained-AST differential dual run. The Vapor selector
and its thread-local access are absent from ordinary builds. Its guard restores
the caller's selection after nested scopes and unwinding, covered by a focused
test. The existing explicit Vapor retained option is preserved.

## Measurement and adoption

Source I/O and hashing happen before timing. Fixture ids are actual
repository-relative paths (`tests/_fixtures/...`), so local imported types
resolve against the checked-out fixture files. The workflow runs from the
repository root. Dependency packages are not installed: unavailable external
types are outside the successful imported-type coverage. Each timed pass includes SFC
parsing, script/template/style compilation, module assembly, and owned result
destruction. The profiler is disabled. Each lane has one warmup batch followed
by nine alternating paired batches, each containing five complete corpus
passes. The retained scope is entered outside its timing window. The runner
build uses `ci-opt`: optimization level 3, thin LTO, 16 codegen units. These
measurements describe that profile, not release-profile performance. The example
uses mimalloc, like the default native CLI, without allocation tracking.

Before timing, a separate profiler-enabled pass reads the selected and forced
retained verdict for every input and enforces one selection counter at most.
Forced retained must never report native acceptance. The SSR scoped selector
uses `LegacyOnly`, which emits no selection counter; that absence is recorded,
not presented as an observed retained verdict. A separate known-admitted
selector probe requires native acceptance before forcing and the backend's
expected retained accounting afterwards (no S4 selection counter for SSR).
Parse errors, inputs without a
template, and diagnostics on either lane (including warnings) are recorded
explicitly. Clean accepted and fallback inputs form separate timing cohorts.
Explicit Vapor sources retain their actual backend identity and a separate
`routed_vapor` cohort under requested DOM shapes. Every report includes all input
paths, source hashes, a deterministic manifest hash, exact head, options,
raw paired timings, output hashes and diagnostic observations. Native adoption
can be recomputed from those observations independently of timing cohorts.

Accepted DOM modules preserve the existing P3-17 code/CSS/diagnostic-message
parity oracle against the forced retained compile. Vapor programs may differ;
output equality, diagnostics and maps are observations, and the independent
runtime and source-map gates remain required. Two additional full-corpus
profiler-enabled passes export attribution for each lane **after** the timed
samples. Attribution times are diagnostic data, not acceptance medians.

A separate `davinci-production-profile` executable records detailed Vapor
bridge stages on runner 1. Its extra `davinci-benchmark-profile` spans are
absent both from ordinary production builds and from the timed-pair executable.
This executable accepts only `--attribution-only`; it cannot emit acceptance
timings. The artifact records S1 parsing, S1-to-S2 and S2-to-S3 lowering,
markup admission, retained-expression indexing, template carriers, text capture,
generic verification, native admission, and IR projection separately.

The separate attribution build wraps mimalloc in `ProfilingAllocator` and
exports actual allocation calls and requested bytes. Timed builds retain plain
mimalloc. A feature restricted to the attribution build measures the S1-to-S2
provenance record span and counts the fully retained records, vector capacity,
and heap-backed CompactString capacities at stage finish. These capacities
describe retained storage, not allocation traffic or peak memory. The record
span excludes caller-preformatted `after` strings; the enclosing S1-to-S2 span
includes them. Transform-pass provenance appended later is outside the finish
counters. These instrumented profile times are not acceptance medians.

## Acceptance limits

The corpus excludes hydrated `_git` projects, `_git-worktrees`, and
`node_modules`. It covers committed SFC fixtures and real shipping settings;
it does not claim the entire production project matrix or universal speed.
The all-input cohort mixes admitted and fallback routes, so it cannot alone
establish native-route improvement. Interpret the admitted cohort and exact
adoption table alongside it. All three independent runner results and their
raw arrays must be retained, including failures and slower observations.

This protocol changes no numeric budget and closes no phase checkbox. The
P0-3 retained transform/lower/generate ladder remains a separate baseline;
its historical numeric baseline is not inferred from this whole-SFC harness.
P3-6 and P3-16 remain open until their semantic, runtime, map, adoption and
fixed performance acceptance conditions are actually met.

## Final Transition control observation

[Run 36241061949](https://github.com/ubugeeei-prod/vize/actions/runs/36241061949)
completed successfully on benchmark head
`43179ebfd0b77a72b625dcd8efafd17e5fe82e29`, whose production parent is
`d424f46d0401575a591e34272aae2c28298d4901` (the Transition stack layer).
The manifest contains 442 SFCs and hashes to
`673fc2d888d7f22d3fc19c2da64103d273bdc4a27bbf8e4356c9c58169c96a70`.
All three `davinci-production-perf-*` artifacts retain raw paired samples,
input observations and runner provenance; runner 1 also retains separate
detailed bridge attribution. `davinci-production-summary` validates and
aggregates those reports.

The committed [raw timing record](./p3-6-production-control.samples.json)
preserves every cohort's paired arrays and all three runner provenance dumps
beyond artifact expiry. It pins the exact raw-report hashes and the shared
per-input observation hash; full per-input records and separate detailed
attribution remain in the run artifacts.

Ratios below are selected divided by retained. Each row retains the three
independent runner ratios; the center is their median. Clean accepted cohorts
exclude diagnostics on either lane. These controlled observations show slower
selected compiles in every requested shape and **do not satisfy P3-6 performance
acceptance**. No budget or acceptance condition was changed.

| Shape      | Clean accepted files | Median ratio | Three runner ratios    | All-input median |
| ---------- | -------------------: | -----------: | ---------------------- | ---------------: |
| DOM inline |                  335 |       1.0298 | 1.0298, 1.0308, 1.0249 |           1.0698 |
| DOM module |                  335 |       1.0295 | 1.0302, 1.0295, 1.0211 |           1.0629 |
| SSR        |                  335 |       1.3345 | 1.3357, 1.3345, 1.3258 |           1.3242 |
| Vapor      |                  207 |       1.0654 | 1.0699, 1.0654, 1.0567 |           1.1470 |

The 128 clean Vapor fallback inputs have median ratio 1.1962, with runner
ratios 1.2043, 1.1962 and 1.1930. This cohort remains separate from admitted
native work when choosing a bounded optimization trial.

There are 351 compiled templates in each shape. Observed native acceptance
counts are 347 DOM inline, 347 DOM module, 349 SSR and 208 Vapor; the remaining
counts are 4, 4, 2 and 143 respectively. These observations preserve the fixed
P3-17 floor rather than replacing its gate. No diagnostic-message differences
were observed. Clean accepted DOM/SSR pairs have no code differences; 23 Vapor
pairs have different code. This comparison uses default maps disabled and
does not establish TS-31 source-map or TS-33 runtime parity.

## Repaired retained walk accounting control

[Run 36242692012](https://github.com/ubugeeei-prod/vize/actions/runs/36242692012)
completed successfully on exact benchmark head
`887cc91a7c421dacfa6dd9dfcf7517e01ed7d784`, production parent
`9eb3b0413857cecb3c17bd5c8ae9f46a4b60279c`. That parent repairs retained
conditional-slot metadata node visit accounting, preserving the historical
walk floor. Its atomic probe increments are active in the timed builds, so
this is a new control rather than an assumed equivalent timing revision.
The [raw timing record](./p3-6-production-repaired.samples.json) retains all
paired arrays and runner provenance. Manifest, adoption and complete per-input
output/diagnostic observations match the earlier control.

| Shape      | Clean accepted files | Median ratio | Three runner ratios    | All-input median |
| ---------- | -------------------: | -----------: | ---------------------- | ---------------: |
| DOM inline |                  335 |       1.0244 | 1.0204, 1.0244, 1.0295 |           1.0647 |
| DOM module |                  335 |       1.0233 | 1.0233, 1.0219, 1.0263 |           1.0603 |
| SSR        |                  335 |       1.3345 | 1.3331, 1.3345, 1.3357 |           1.3220 |
| Vapor      |                  207 |       1.0673 | 1.0629, 1.0681, 1.0673 |           1.1486 |

The 128 clean Vapor fallback inputs have median ratio 1.2001, with runner
ratios 1.2001, 1.1980 and 1.2010. These observations still do not meet P3-6
performance acceptance. The fixed gates and the earlier slower evidence remain
unchanged.

## Rejected expression-reuse trial

[Run 36242833278](https://github.com/ubugeeei-prod/vize/actions/runs/36242833278)
completed successfully on trial head
`82778683f42e88673e97e6e7237a790eb178726f`, using the repaired control above.
The trial retained four recent successful JavaScript ASTs keyed by exact bytes,
while keeping fresh authored wrappers and spans. Parser refusals, eviction,
dialect and prefix settings, complexity coordinates and decoded Vapor maps
passed the focused trial checks. The complete input output/diagnostic
observations equal the repaired control.

| Shape      | Control median | Trial median | Trial runner ratios    |
| ---------- | -------------: | -----------: | ---------------------- |
| DOM inline |         1.0244 |       1.0245 | 1.0111, 1.0355, 1.0245 |
| DOM module |         1.0233 |       1.0259 | 1.0259, 1.0353, 1.0216 |
| SSR        |         1.3345 |       1.3300 | 1.3300, 1.3302, 1.3135 |
| Vapor      |         1.0673 |       1.0664 | 1.0664, 1.0667, 1.0603 |

The runner ranges overlap the control. No meaningful improvement was measured,
so the expression-reuse implementation remains a rejected trial and is not
included in the production branch. The [raw trial record](./p3-6-expression-reuse.samples.json)
preserves the paired arrays and provenance without changing any fixed gate.

## Exact-main 561-SFC control

[Run 40853274181](https://github.com/ubugeeei-prod/vize/actions/runs/40853274181)
completed successfully on exact `main`
`44f1aff002c56b932c9be27f225f7b844ab16290`. The workflow validated dispatch
base `f726433219b3369151efa8a15f26cecde3b4a9a2`, exact head, and
`production_only=true` before measuring. Its manifest contains 561 committed
SFCs and hashes to
`2074f89546701138e83585db2274be9ecbd535f4cc01fef30c1ec60201699011`.

The [compact timing receipt](./p3-6-production-main-2026-10-29.samples.json)
pins the four immutable artifact ids and archive digests, the hashes of all
downloaded files, and the three runner records. Each runner record preserves
all nine alternating paired arrays for every cohort and the full Linux, CPU,
Rust and LLVM provenance. The 2.8 MiB reports remain in their immutable
artifacts rather than being duplicated in Git. The canonical input-observation
hash is identical on all three runners.

Ratios remain selected divided by retained. These are report-only observations;
they neither change a budget nor close an acceptance gate.

| Shape | Clean accepted files | Median ratio | Runner range | Three runner ratios |
| --- | ---: | ---: | --- | --- |
| DOM inline | 462 | 1.0703 | 1.0690–1.0716 | 1.0690, 1.0716, 1.0703 |
| DOM module | 462 | 1.0677 | 1.0598–1.0699 | 1.0677, 1.0598, 1.0699 |
| SSR | 462 | 1.3294 | 1.3277–1.3375 | 1.3375, 1.3294, 1.3277 |
| Vapor | 276 | 1.0655 | 1.0617–1.0751 | 1.0655, 1.0617, 1.0751 |

There are 467 compiled templates. DOM inline and module select the requested
emitter for 462, SSR for 464, and Vapor for 276. Vapor leaves 191 templates
fallback, rejected, or unrecorded. The clean accepted Vapor pairs contain 27
code differences and no diagnostic-message differences. Default maps are off,
so this run does not establish source-map parity. Emitter selection also does
not establish native-only fact production and cannot satisfy #6853. The Vapor
all-input and fallback medians are 1.1475 and 1.1964 respectively.

The prior repaired 442-SFC control measured 207 accepted Vapor inputs at
1.0673. The new 1.0617–1.0751 runner range overlaps that control; the larger
manifest therefore supplies fresher coverage, not evidence of a meaningful
speed improvement.

### Bridge attribution and next bounded trial

Runner 1 also produced a separate profiler-enabled attribution build. Its
[compact attribution receipt](./p3-6-production-main-2026-10-29.attribution.json)
pins the raw 1,431,930-byte report and retains the exact accepted-cohort bridge
spans, allocation traffic and provenance counters. These diagnostic spans are
not the timing samples above.

Across the 276 accepted Vapor inputs, selected attribution requested
53,599,067 bytes in 47,116 allocations versus retained's 51,052,349 bytes in
48,506 allocations. The native bridge stages account for 1,874,617 ns of self
time and 4,941,023 self-allocated bytes. The four largest self-time stages are:

| Stage | Self time | Total allocation |
| --- | ---: | ---: |
| S1 to S2 | 703,393 ns | 664,998 bytes |
| native admission | 380,396 ns | 32,240 bytes |
| S1 parse | 270,729 ns | 4,550,320 bytes |
| IR projection | 209,886 ns | 42,148 bytes |

S1 parse plus S1-to-S2 lowering account for 1,062,625 ns of attributed total
time and 5,215,318 bytes, 98.6% of bridge allocation traffic. The next bounded
optimization trial should therefore be limited to the accepted Vapor surface
bridge: measure an admission-oriented S1-to-S2 representation that avoids
materializing unconsumed storage while retaining every diagnostic, text fact,
template-carrier fact, and adverse provenance witness the current admission
reads. It must keep selector decisions, accepted/fallback membership, generated
bytes, diagnostics, runtime behavior, maps, capture output and full-provenance
tooling unchanged. If those invariants require a broader production-lane
switch, the trial stops rather than widening scope.

This control still fails P3-6 performance acceptance. Its source-map, runtime,
native-only adoption, zero-fallback and fixed-budget gates remain independently
open.
