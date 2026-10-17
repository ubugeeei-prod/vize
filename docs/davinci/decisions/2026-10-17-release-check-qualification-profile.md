# Separate release qualification from instrumented source measurement

## Decision

An explicit `release-qualification` Check dispatch profile runs every existing
correctness lane, including the seven required full jobs, twelve published-crate
SemVer checks, real editor host, all tooling shards, full Rust workspace tests and
doctests. Only the instrumented `source-coverage` trend job is omitted in this
profile. The default is `false`; daily and ordinary manual Full Check continue
to run source coverage with the same tests, thresholds, native packets and report.
Fixture coverage and nightly branch coverage remain in both profiles.

The existing release gate dispatch plan opts into this profile. It still binds
the actual H, release ref, workflow, event, latest run and attempt. Publication
still requires the entire Check workflow to finish successfully and all nineteen
named jobs to succeed. Pending, failed, cancelled and duplicate evidence remain
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
