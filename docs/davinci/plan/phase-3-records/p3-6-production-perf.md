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

Source I/O and hashing happen before timing. Each timed pass includes SFC
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
