# Separate release qualification from instrumented source measurement

## Decision

An explicit `release-qualification` Check dispatch profile runs every existing
correctness lane, including the source-bound full roles, twelve published-crate
SemVer checks, real editor host, all tooling shards, full Rust workspace tests and
doctests. Only the instrumented `source-coverage` trend job is omitted in this
profile. The default is `false`; daily and ordinary manual Full Check continue
to run source coverage with the same tests, thresholds, native packets and report.
Fixture coverage and nightly branch coverage remain in both profiles.

The existing release gate dispatch plan opts into this profile. It still binds
the actual H, release ref, workflow, event, latest run and attempt. Publication
still requires the entire Check workflow to finish successfully and every
source-bound named job to succeed. Pending, failed, cancelled and duplicate evidence remain
refusals. The profile does not reuse C or a previous source's Check evidence.

The profile is restricted to `release/vSEMVER` branches and `vSEMVER` tags. Its
explicit validation refuses other or malformed refs. The operator separately
authenticates the source pin and frozen branch before and after qualification;
the input does not establish those identities or authorize publication. Ordinary
manual runs on a release branch or tag keep source coverage unless the caller
explicitly selects this qualification profile.

This is a policy successor to the [source-coverage nextest decision](./2026-10-13-6830-source-coverage-nextest.md),
which retained instrumented coverage as an implicit release blocker. The seven
named jobs alone did not previously exempt it: whole-workflow success included
coverage. A release-profile run supplies no instrumented source-coverage result
for H. Component coverage measurement continues on daily and ordinary manual
Full runs; it must not be inferred from this profile's success.

## Preserve the Rust gate while running its producers concurrently

`clippy-and-test` becomes a strict aggregate of three independent producers:
Clippy plus no_std portability, Davinci allocations, and the native differential
tail. Each retains the original setup and runtime settings. Every original task,
argument, feature/backend selection, 100 Nuxt iterations and artifact retention
step is preserved. The native tail still receives `workspace-already-tested:
true`; the independent full-workspace execution and original `full-rust-report`
inventory verification remain required.

The aggregate runs after all three producers and refuses any failed, cancelled,
skipped or unresolved dependency. Its required job name remains
`clippy-and-test`, and `full-rust-report` continues to require it together with
`full-rust-workspace`. No successful aggregate can replace missing workspace
inventory, JUnit, doctest or source identity evidence.

All three runners read independent clones of the existing `clippy-test` cache
role. That role remains restore-only for target uploads; introducing new cache
roles must not accidentally restore the former oversized target upload problem.
The frozen original job fixture tests the complete setup and ordered task
vectors, including literal command bytes, against the three new producers.

## Event scope

| Event and ref                                  | Explicit profile | Source coverage                             | Other full lanes                        |
| ---------------------------------------------- | ---------------- | ------------------------------------------- | --------------------------------------- |
| Daily schedule, main                           | false            | Runs                                        | Run                                     |
| Ordinary manual, main or feature branch        | false            | Runs                                        | Run                                     |
| Ordinary manual, release branch or version tag | false            | Runs                                        | Run                                     |
| Manual, valid release branch or version tag    | true             | Omitted                                     | All run, including twelve SemVer checks |
| Manual, other or malformed ref                 | true             | No accepted qualification; validation fails | Workflow cannot succeed                 |
| PR, merge group, push                          | absent           | Existing behavior unchanged                 | Existing iteration scope unchanged      |

## Evidence and limits

At source `06a51573858160eca1dfe4d54a06f90812f0f6b9`, Full Check
[39000354748](https://github.com/ubugeeei-prod/vize/actions/runs/39000354748)
ran the original serial Rust job for 16m43s. Setup took 47s, Clippy 2m20s,
portability 12s, allocation gates 3m33s and the native tail 9m45s. Concurrent
execution removes their serial dependency; cache misses and runner scheduling
still require measurement. Source coverage completed in 17m58s in that run;
without the parallel split, removing it would leave the 16m43s serial Rust
path. The unchanged JS package test job took 13m09s, so the combined change
also cannot establish a ten-minute qualification bound. The existing run is
not changed or reclassified.

This does not establish the fifteen-minute release target. In original Release
[38805261385](https://github.com/ubugeeei-prod/vize/actions/runs/38805261385),
the retained successful Windows ARM native build lasted 17m17s, and native npm
publication lasted 11m27s before its downstream publication stages. Those are
separate remaining critical paths. Moving H qualification before Release R
does not remove it from elapsed release-command time. Qualification, R and
installed acceptance timings must continue to be reported separately and in total.

## Overlap qualification with artifact builds

The official operator starts or finds exact-H Check Q before dispatching Release R,
then authenticates the unchanged candidate again. It does not call a pending Q
qualified. R's existing native, package and editor build fanout runs while its
read-only preflight joins that already existing Q. The original ten-minute
`--verify-only` step follows the join; crate validation remains independent.
`candidate-ready`, metadata, blockers, promotion, tag and publication gates remain
unchanged. No Q dispatch, bootstrap or rerun capability exists in R's join.

The operator authenticates H's immutable dispatch declaration before selecting
this protocol. A known legacy H without the deadline input retains its original
blocking Check qualification and receives no undeclared input. Unknown or
ambiguous declarations refuse before Q or R dispatch. Resuming an existing R
continues to reuse that R's original source and inputs.

The operator carries one absolute qualification deadline into R: the earlier of
its inherited effective deadline and Q's authenticated creation time plus the
existing 345-minute full-gate cap. Polls and jobs cannot restart that budget.
Missing, failed, cancelled or changed Q evidence refuses. An expired pending Q
cannot wait again. An official resume may immediately authenticate an already
successful latest Q and every source-bound required job, without renewed
waiting or dispatch. The unchanged verify-only step rechecks live evidence before
publication. A manually dispatched new candidate must supply a valid bounded
deadline; it cannot extend the authenticated Q cap.

This removes the serial Q-then-R build dependency. It does not establish a
fifteen-minute release: the historical Windows ARM build and native npm publish
critical paths above remain separate optimization work. Actual-source Q/R overlap,
total release duration and installed acceptance still require measurement.

## Reuse exact-H product execution at official integration

The closed `full-qualification.yml` callee preserves all seventeen original
full-only jobs and adds the existing Nuxt build/SSR consumer. The actual run's
GitHub `referenced_workflows` record must bind this callee to the same repository,
H, and release ref. This profile requires eight actual successful full roles:
Tooling, JS package aggregate, JS build, full JS report, workspace tests/doctests,
Clippy aggregate, full Rust report, and Nuxt consumer. The twelve SemVer jobs are
unchanged. A reference alone grants no success. Legacy runs without this callee
retain their original seven-role contract and cannot use the product reuse path.

The existing integration catalog still authenticates the source, pin, operator
lease, latest whole-success Q and release readiness. Its strict v1 receipt and
legacy resume behavior remain unchanged. An adjacent product proof additionally
checks raw Git commit object hashes, H's sole parent C, the merge candidate's
actual sole parent, and equality of their complete tree IDs. Shipping-catalog
identity alone is insufficient. A valid different tree or legacy Q runs the
ordinary product checks; malformed authority or changed evidence refuses.

Only the duplicate archive plus five ordinary Rust workspace cohorts and Nuxt
consumer may use this proof. Q's real workspace execution now captures all
registered typechecker fixture cases and the default-feature TS40 projections.
An explicit workspace receipt binds the source/runtime, original raw inventory,
actual pre-run executable SHA, post-run SHA, complete JUnit, ordered diagnostics,
public results and all projection bytes. The full report checks the actual
successful producer attempt's immutable artifact; a retained successful job is
not claimed as a new execution. Existing archive-v1 receipts and the five real
worker aggregation remain the unchanged ordinary route.

Clippy, native lexer parity, isolated feature/differential jobs, JS/conformance,
real native hosts, playground and canonical observers still execute in the
queue. They start beside qualification rather than waiting for it. The strict
source/product reports and required final Check report freshly verify the same
live authority before accepting any reused result. Every ordinary PR, unknown
candidate or unsupported source keeps its actual product route. None of the
four required Check contexts is removed or made advisory.

These changes target duplicate build and test work. The first same-tree official
integration must earn real hosted execution and reuse evidence; pure fixtures
are not native qualification or a measured speedup. Pre-Q startup, publication,
registry visibility, JSR consumers and installed acceptance remain part of total
release time. The fifteen-minute end-to-end target remains unearned.
