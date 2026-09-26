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
