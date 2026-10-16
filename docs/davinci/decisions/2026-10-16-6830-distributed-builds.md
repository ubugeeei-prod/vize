# CI caches and distributed builds

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

This record selects the cache and build-system strategy for the October 16 CI
optimization. The targets are at most five minutes for PR feedback and about ten
minutes for a release. They are acceptance targets, not measured results of this
change. Bazel, Buck2, a remote compiler cache, and remote execution are evaluated
below; none is enabled by this record.

## Measured starting point

The baseline is main `b4be3b70977b80a689c8e5ada62d05283e86b868`. Actions job
timelines were inspected on October 16, using first-attempt timestamps so a retry
cannot make an old build appear faster.

| Scope             | Evidence                                                                                                                                                                                                                                    | Elapsed time          | Dominant work                                                                  |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------- | ------------------------------------------------------------------------------ |
| Main Check        | [38808071557](https://github.com/ubugeeei-prod/vize/actions/runs/38808071557)                                                                                                                                                               | 14m49s                | JS package tests: 13m30s inside a 14m19s job                                   |
| Passing PR Checks | [38805236717](https://github.com/ubugeeei-prod/vize/actions/runs/38805236717), [38805184162](https://github.com/ubugeeei-prod/vize/actions/runs/38805184162), [38804888129](https://github.com/ubugeeei-prod/vize/actions/runs/38804888129) | 12m03s–12m51s         | 48 executed jobs; original-project Oxlint transport: 7m38s–8m09s               |
| Release attempts  | [38805261385](https://github.com/ubugeeei-prod/vize/actions/runs/38805261385), [38798022260](https://github.com/ubugeeei-prod/vize/actions/runs/38798022260), [38791695153](https://github.com/ubugeeei-prod/vize/actions/runs/38791695153) | 40m16s–45m55s, failed | Windows native build step: 11m48s–18m57s; candidates ready after 20m59s–27m51s |

These release timings describe failed first attempts, not successful publication
latency. The early release safety/publish-plan checks took 20–113 seconds in this
sample. Moving those checks alone cannot deliver a ten-minute release. The PR
sample consumed 108–130 runner-minutes per run, so splitting everything into more
jobs without narrowing the affected work would also increase cost.

The repository already has Cargo dependency graphs, Vite+ task inputs, Nix
derivations, and Blacksmith target disks. Its Rust cache policy allows trusted
default-branch jobs to commit stable snapshots and gives candidates disposable
clones. Source blob checks restore mtimes only when content matches the recorded
tree. Preserve those correctness properties while removing redundant work.

## What the larger build systems actually do

Google describes an action graph with declared inputs, deterministic outputs,
remote action-result storage, and workers that run independent ready actions.
Fetching artifacts lazily avoids transferring every intermediate file. It also
explicitly warns that a cache is useful only when downloading costs less than
rebuilding. These are architectural properties to adopt; Google's reported
infrastructure and scale are historical examples, not a sizing estimate for
Vize. [Software Engineering at Google, build systems](https://abseil.io/resources/swe-book/html/ch18.html).

Google's CI model keeps fast, reliable tests in presubmit and runs broader
affected tests asynchronously after submission. This requires prompt handling of
post-submit failures; moving a test later does not remove responsibility for its
result. That supports a bounded product-focused PR lane and a scheduled real
project matrix. [Software Engineering at Google, continuous integration](https://abseil.io/resources/swe-book/html/ch23.html).

Cloudflare's public `workerd` repository supplies a concrete Bazel example. At
`9d8ec3907f62fe5240f3dd40a5c1fcbb1251c7eb`, its reusable CI workflow uses an HTTP
remote cache for build and test actions. It deliberately avoids broad disk-cache
restore prefixes because accumulated cache contents increase transfer time.
This is evidence about `workerd`, not a claim about every Cloudflare repository.
[Cloudflare's Bazel workflow](https://github.com/cloudflare/workerd/blob/9d8ec3907f62fe5240f3dd40a5c1fcbb1251c7eb/.github/workflows/_bazel.yml).

The same repository sets `--jobs=64`, downloads minimal remote outputs for tests,
records build/test profiles, and limits expensive test configurations. Release
jobs fetch the distributable binary while leaving other intermediates remote.
Those settings show how caching, measurement, and test selection work together;
the number 64 is not a recommended Vize shard count.
[Cloudflare's CI configuration](https://github.com/cloudflare/workerd/blob/9d8ec3907f62fe5240f3dd40a5c1fcbb1251c7eb/build/ci.bazelrc).

## Decision for this optimization

Improve the existing Cargo/Vite+/Actions graph first. Its current critical path
includes broad consumer tests, repeated runtime preparation, and serialized
artifact preparation; replacing Cargo alone does not address all three.

- Select affected product lanes from the changed files and transitive dependency
  closure. Include generators, fixtures, configuration, and lockfiles in that
  graph. Unknown inputs must conservatively select the broader lane.
- Keep compiler, type checker, linter, formatter, native binding, and packaging
  correctness checks relevant to a change. Run the broad real-project matrix,
  documentation consistency, and repository-maintenance verification outside
  the everyday product feedback path.
- Produce a source/native artifact once per compatible platform and build
  configuration, then fan out its consumers. Shard independent tests using
  observed durations. A shard that recompiles the entire workspace can cost more
  than it saves.
- Keep linked Cargo outputs on stable Blacksmith target disks. Healthy mounts
  should not additionally download and upload a whole target archive. Separate
  incompatible target triples, profiles, features, and compiler flags; let Cargo
  determine source/dependency freshness inside a compatible target.
- Use trusted main builds to warm reusable namespaces. Release candidates can
  read compatible snapshots without gaining permission to commit them. A cache
  miss must build normally and cannot bypass artifact validation.
- Run TypeScript orchestration directly with the supported Node runtime. Cache
  Vite+ bootstrap runtimes and package-manager data without a network lookup for
  the latest runtime on every job. Continue validating the configured runtime
  and frozen dependency lockfile.
- Preserve lock-scoped Nix store reuse and run Nix qualification when its inputs
  change or in broader qualification. Nix reproducibility does not require
  every unrelated JavaScript job to build the Nix development environment.

Blacksmith sticky disks clone the last committed snapshot onto each runner.
They are persistent filesystem caches, not an action-result cache or a remote
compiler farm. Consequently their usefulness depends on build-tool freshness
checks and stable workloads, not merely a successful mount.
[Sticky-disk architecture](https://github.com/useblacksmith/stickydisk#architecture).

## Alternatives and migration cost

| Option                                | Useful property                                                        | Cost or limitation                                                                                               | Status                                 |
| ------------------------------------- | ---------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | -------------------------------------- |
| Cargo + Vite+ + sticky targets        | Reuses the current build graph and linked outputs; no new service      | Correct task inputs and compatible namespaces remain essential                                                   | Selected for this PR                   |
| sccache with S3/R2 or another backend | Shares eligible compiler outputs across target directories and runners | Does not replace linked target caches; backend authorization, retention, transfer cost, and measurement required | Evaluated; not configured              |
| Bazel + remote cache                  | Caches declared compile, generation, link, and test actions by inputs  | Rust, JS, NAPI, WASM, generators, and packaging need explicit rules/toolchains                                   | Proposed follow-up pilot; not migrated |
| Buck2 + remote cache/execution        | Incremental action graph and Bazel-compatible remote execution API     | Another rule/toolchain migration with the same integration and operational work                                  | Evaluated alternative; not adopted     |
| Remote execution workers              | Distributes independent cache misses across machines                   | Worker platform, scheduler, capacity, credentials, observability, and operating cost                             | Not provisioned                        |

An sccache prototype should complement the sticky target cache. Its documented
Rust restrictions include disabled incremental compilation, no caching for
crates invoking the system linker (`bin`, `cdylib`, and `proc-macro` among them),
and caveats for procedural macros reading files. The native addon and CLI still
need their link/build artifacts. Compare end-to-end time and cache eligibility
before installing another cache into every job.
[sccache Rust support](https://github.com/mozilla/sccache/blob/main/docs/Rust.md).

Sccache documents S3-compatible and Cloudflare R2 configuration, including an
S3 read-only mode. R2 is a possible storage backend, not proof of Cloudflare's
internal Rust build practice. No bucket, credentials, or remote cache service is
created by this change.
[sccache configuration](https://github.com/mozilla/sccache/blob/main/docs/Configuration.md).

Bazel separates the action cache from a content-addressed store of output bytes.
It can reuse fine-grained outputs across machines, but a single rule wrapping
`cargo build --workspace` would still be one coarse action. A useful pilot needs
crate-level rules and declared build-script/generated-file inputs, preserving
Cargo features, vendored/git dependencies, target platforms, and test data.
`rules_rust` offers Cargo-based dependency generation, reducing rather than
eliminating that migration work.
[Bazel remote caching](https://bazel.build/remote/caching),
[Rust crate universe](https://bazelbuild.github.io/rules_rust/crate_universe_bzlmod.html).

Blacksmith already offers a Bazel/Buck2 remote cache behind an organization-admin
dashboard setting. It caches across branches and PRs and does not supply remote
execution. Its action-key matching must not be confused with authorization for
a contributor to publish trusted results. Before a pilot uses that service,
verify read-only candidate access and trusted-writer isolation; repository or
architecture separation alone is insufficient. The setting and its availability
for this organization have not been verified or changed.
[Blacksmith Bazel caching](https://docs.blacksmith.sh/blacksmith-caching/bazel-build-caching).

Buck2 supports the Bazel remote execution API, so a pilot need not assume a
proprietary worker protocol. It still requires explicit execution platforms and
service configuration. The comparison should use the same Rust workload and
artifact tests, not unrelated headline build times.
[Buck2 remote execution](https://buck2.build/docs/users/remote_execution/).

## Follow-up acceptance

Measure checkout/setup, cache restore, compilation/linking, test execution,
artifact transfer, queue delay, and publication separately. Report the entire
workflow elapsed time as well as the critical path and runner-minutes. Keep cold
and warm runs separate and compare equivalent changes; three old PRs are a
baseline sample, not a dependable p95 estimate.

The optimization is successful only after the actual PR and release workflows
meet their targets with product checks intact. Warm-cache predictions and an
earlier candidate-ready timestamp do not establish a ten-minute publication.

If warmed builds still miss those targets because compilation dominates, the
next experiment is a bounded Bazel pilot for one expensive Rust product and its
tests. Compare cold build, unchanged rerun, one leaf edit, shared dependency
edit, and lock/toolchain edit against Cargo. Verify diagnostic and packaged
artifact equivalence, collect action-cache hits and transferred bytes, and
demonstrate that candidate jobs cannot publish trusted cache entries. Promote
the pilot only if the end-to-end improvement pays for its maintenance and
service cost. Remote execution remains a separate experiment after the action
graph and cache are correct.
