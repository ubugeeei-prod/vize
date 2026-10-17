# Compile only the Rust targets a PR cohort needs

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The named Rust cohorts split execution, but nextest's `-E` filter does not
narrow Cargo compilation. On real Rust source PR #9199,
[Check 38875549893](https://github.com/ubugeeei-prod/vize/actions/runs/38875549893)
took 6m41s. The Language server job took 330s: 172s compiling, 114s testing,
and 44s in setup, listing and artifact work. Its 1,701 tests used 53 binaries;
another 1,152 binaries were skipped after the build. The target snapshot was
already warm: mounting took 342ms and source freshness restored 31,058 files
while invalidating the 25 changed files. Adding another remote cache does not
address this repeated source-dependent build.

## Bounded target projection

`run-rust-test-cohort.ts` reads current offline Cargo metadata and the existing
semantic cohort registry. It preserves every affected `--package` argument,
including dev/build dependency feature unification, and retains shared library
and binary unit targets. It adds explicit integration target names only for
the current cohort. The existing nextest filter still decides which tests run.
The current workspace's Language server cohort needs 52 integration targets
instead of the workspace's 1,191 candidate integration targets.

Cargo remains authoritative for cases that need its default target selection:

- Any cohort with a feature-gated integration target keeps the full build.
  Explicit `--test` would otherwise fail when its required feature is disabled.
  Names are resolved across all selected packages, including duplicate names.
- Disabled library/binary targets, runnable examples/benches and unfamiliar
  target kinds keep Cargo defaults. Invalid or missing metadata fails the job.
- At least one existing cohort retains the default build whenever selected
  packages contain examples. This preserves compile-only examples without
  changing their harness or adding another command or job.

This intentionally optimizes the measured Language server path first. The
current compiler, typechecker, lint and CLI cohorts retain default builds
because they contain feature-gated integration targets. Future targets are
discovered from metadata; a new gated target conservatively restores the full
build instead of disappearing from coverage. Package ownership and CLI name
prefixes have one authority in `rust-test-cohorts.ts`.

Clippy, doctests, differential feature lanes, runtime environment flags,
nextest profiles, JUnit/artifact identities and all aggregate guards remain
unchanged. The merge queue continues to build and verify its complete
source-bound archive and full test profile.

## Validation and acceptance

Focused Node tests cover future targets, duplicate target names, unavailable
features, library kinds, binary-only/test-only/benchmark-only packages,
compile-only examples and malformed metadata. The existing workflow execution
test exercises the actual PR command and preserves its complete package list,
profile, filter and exclusive runtime flag; the queue command remains intact.
An isolated Cargo workspace also verified that both another selected package
and a benchmark-only package continue to activate library features while
unrelated integration binaries are excluded. A separate probe confirmed that
`--bins` skips unavailable feature-gated binaries as the default build does.

Run the focused contracts with:

```sh
node --test tests/tooling/rust-test-cohorts.test.ts tests/tooling/rust-pr-runtime-tier.test.ts
```

The five-minute target is not yet demonstrated for real compiler edits.
Qualification must compare the projected PR inventory against the full queue
inventory at the same source head, accounting only for the unchanged PR
profile exclusions, then measure the actual Actions critical path. This change
does not flatten the three report layers or claim an unmeasured speedup.

References: [Cargo target selection](https://doc.rust-lang.org/cargo/commands/cargo-test.html#target-selection),
[nextest build reuse and filtering](https://nexte.st/docs/ci-features/archiving/).
