# Observe performance on every pull request

Performance changes need an attributable base/head comparison while normal development remains free to merge without waiting for benchmark runners. `pr-performance-observation.yml` runs independently on every opened, updated, or reopened PR, including contributions from forks. It is not a required status check, a dependency of Check, a merge-group job, or a release gate. Existing historical benchmark budgets retain their own behavior.

The optional **Performance observations** check links to an Actions summary and a downloadable `pr-performance-<PR>-<head SHA>-<attempt>` artifact. Reports, raw invocations, source identities, build receipts, and inputs are retained for 90 days. A subsequent push cancels the superseded measurement for that PR. An explicit dispatch can repeat any immutable base/head pair using the selected workflow driver.

## What is compared

The first automatic campaign uses the existing public 1,000-SFC generator and the existing compile, lint, format, and type-check lane catalogue. It records five adjacent base/head pairs, reversing execution order on alternating pairs, after one warmup pair. Both binaries run on the same eight-vCPU Linux worker with one physical Vue/native TypeScript dependency provider. Single-worker and default-pool lanes remain separate.

The report records each sample, paired change, observed regression candidates, binary byte sizes and SHA-256 identities. Memory is measured separately from latency with GNU time. Its maximum resident set size is a maximum for a process/waited child; it is not the sum of simultaneous Rust, Node, and Go memory. It must not be presented as an aggregate whole-project memory peak.

These are fresh-process measurements after warmup. Filesystem pages and persistent product storage may be warm. This campaign does not establish a no-cache cold-start result or the 2,500-SFC/500-ms target. The separate source-allocation, instruction, cold-scale, mapper, and LSP campaigns retain their explicit scopes. Instrumented timings cannot replace normal-binary timings.

## Source and measurement custody

The measurement driver is checked out separately at the immutable workflow SHA. Its tools, dependency provider, and corpus do not depend on whether an older PR head contains the new measurement infrastructure. Manual replay can therefore measure commits from before instrumentation was introduced. Each product role still checks out the exact PR base or authored head SHA. The source-pair receipt records the driver/tree, merge base, and changed paths. A PR comparison never silently substitutes a historical commit or the synthetic merge commit for either role.

Base and head build sequentially into distinct, initially absent target directories. No compiled target or binary cache is restored. Cargo JSON binds the actual executable to its source role, fresh compilation, features, compiler recipe, toolchain, and binary hash. Both builds use the same optimized `ci-opt` settings with thin LTO, 16 codegen units, stripped symbols, and no profiling feature. This is a consistent comparison profile; its byte size is not a claim about every published platform artifact.

Latency records preserve arguments, selected environment, output, and exit status. Type-check comparisons require invalid and repaired controls, complete corpus coverage, and consistent diagnostics before publishing a speed ratio. A crash, timeout, missing backend, missing diagnostic, or changed output is recorded as unavailable, not as a faster result. Regression candidates remain visible in the summary and JSON without causing a performance-threshold failure.

The workflow has read-only repository permissions and does not post PR comments, alter branch rules, or invoke the existing budget-enforcement job. Measurement/build errors remain visible in the optional check and retained evidence. They do not join the four required repository checks.
