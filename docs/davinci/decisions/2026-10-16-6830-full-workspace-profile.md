# Share the complete Rust workspace build with the trusted ci seed

This is a bounded part of [#6830](https://github.com/ubugeeei-prod/vize/issues/6830),
not completion of the CI optimization umbrella.

At source `d2506ca6b5e31252973f05981829c0367cacf26b`, the scheduled/manual
[full Rust job](https://github.com/ubugeeei-prod/vize/actions/runs/38890449170/job/116938382160)
took 1,848 seconds. Its Test action took 1,445 seconds. The retained
`workspace-timing.json` records 775.357 seconds for nextest, including a
473-second Cargo build (232 external and 43 local crates). Nextest itself reports
290.469 seconds to execute 18,158 passing cases across 1,207 binaries, with 37
ignored cases skipped. Doctests took 34.865 seconds. The approximately 635 seconds
outside those workspace phases includes native and differential **execution** as
well as helper compilation; it is not a build-only saving opportunity.

The full workspace used the default Cargo `test` profile and the `clippy-test`
disk. The queue producer and trusted seed build the workspace with `--profile ci`
on `nextest-ci`. The observed full compiler was Rust 1.99.1, whereas the older
seed pinned 1.99.0. The normal committed compiler is now 1.99.1. A disk namespace
alone cannot remove these compiler/profile fingerprint differences.

Move only the complete workspace nextest and doctest phases to an independent
`full-rust-workspace` job. Match the trusted workspace producer's normal compiler,
Wild 0.9.0 linker, Cargo build parallelism, `nextest-ci` disk, and original
`target` path. Explicitly select Cargo `ci` and nextest `full`. The existing
`ci` profile inherits `dev`: optimization level, debug assertions, and overflow
checks remain the development defaults; debug information and incremental
compilation remain disabled as in the existing producer. This change does not
modify Cargo profiles or instruction-measurement settings.

The receipt labels assertion/overflow defaults as an inference from the committed
Cargo manifest, Cargo config and runtime environment. It rejects profile package,
build, config and environment overrides before making that inference. This is not
a capture of the effective flags of every rustc invocation.

The new workspace job retains Pkl 0.27.2, the pinned Element Plus submodule,
frozen Vite+ dependencies, required TSGO, 100 Nuxt configuration iterations,
nextest 0.9.146, the complete nextest configuration and runtime test groups,
and always-retained result/native observation artifacts. Clippy, portability,
allocation checks, native navigation/SSR, and every feature-enabled differential
recipe remain in the original independent job and cache. A workspace failure
does not prevent that sibling from running. Existing process cancellation
behavior is unchanged.

`full-rust-report` requires successful results from **both actual jobs**. It also
downloads the workspace artifact from the same workflow attempt and verifies
the checked-out commit/tree, compiler, configuration hashes, full command lines,
complete binary/test identities, ignored flags, JUnit execution, and successful
doctests. Missing, failed, skipped, canceled, partial, foreign-source, and
inventory-only results cannot qualify. The caller's
`workspace-already-tested: true` is therefore backed by a required sibling and
source-bound evidence. PR and merge-queue inventory/tier contracts are unchanged.

Every full run inventories and executes the complete `ci` workspace. The manual
`compare-rust-test-profile` input additionally runs the original `test` profile
on the **same final source and runner environment**. It retains separate raw and
normalized inventories, timings, and JUnit for both profiles; the aggregate
requires equal binary/case/ignored identities and complete successful execution
of both workspaces and their doctests. Inventory listing explicitly includes
ignored cases, while execution preserves their existing ignored status.

The comparison defaults to false because a cold original-profile inventory
builds the full debug workspace again. Enable it for the coordinated first
qualification campaign, then use the mandatory `ci` inventory/execution for
recurring full runs. The comparison records build/list time separately from
workspace execution and doctests. It runs the old profile first and `ci` second;
shared runtime assets may be warmer for the second execution, so its timings
must not be described as an isolated causal estimate of profile savings.
The existing serial/nextest benchmark keeps its original default `test` profile.

Local tests exercise subprocess commands and failure propagation with tiny fake
Cargo binaries; they do not compile the real Rust workspace or prove runtime
parity. Workflow contracts preserve the actual fixture/runtime/cache settings,
and receipt mutations reject incomplete inventories, changed source/compiler/
runtime, missing required jobs, truncated JUnit, and absent doctests. Actual
same-head old/new Actions execution, native observations, and end-to-end walltime
remain required before claiming a measured improvement. No new saving is yet
measured by this decision.
