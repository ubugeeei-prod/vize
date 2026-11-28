# Fresco public signal-dispatch probe (source-only)

Prepared under `/tmp`; no product repository files were changed and no build,
native signal run, Callgrind run, timing, Miri run, or package installation was
performed. The Rust and recipe remain uncompiled. Node syntax validation, if
performed by the preparing agent, is not evidence of Rust correctness.

This is supplemental evidence for the saved-action race candidate. It cannot
replace the repository's 110-row shipped-corpus receipt or native terminal and
Miri checks.

## Authored workload

`src/main.rs` calls `vize_fresco::install_terminal_signal_hook` from the actual
base/head library source. A separate named Callgrind window measures that
public installation. Three more windows each deliver one native synchronous
signal through `libc::raise`, the real kernel-installed Fresco handler, its
saved-action lookup, and the actual chained action:

- SIGTERM: an application-owned one-argument handler, preserving SA_RESTART.
- SIGINT: an application-owned SA_SIGINFO handler receiving the real siginfo.
- SIGQUIT: the previously ignored disposition.

The oracle requires exactly one call to each custom handler, the right signal
numbers, live signal information, inherited SIGUSR1 mask membership, and the
entry errno. Each foreign handler deliberately changes errno; the probe also
checks its exit errno survives, while an ignored action keeps the entry errno.
All checks and JSON output occur outside collection windows.

A fresh native `--default-sighup` process tests that Fresco's default branch
restores and re-raises SIGHUP to terminate. It is a correctness oracle, not an
instruction measurement: termination cannot close the marker window normally.

No terminal backend/lease is active. Emergency restoration executes its actual
empty-state paths; terminal I/O and raw-mode restoration remain covered by
existing native tests. There is no induced failed installation, concurrent
retry, or nested-signal execution in this instruction probe. Miri and dedicated
cell tests must establish those claims independently.

## Marker and build identity

`src/instruction.rs` is an exact source copy of
`tools/benchmarks/crates/davinci_harness/src/instruction.rs`. The driver checks
its SHA-256 against the driver, base, and head checkouts before building. It
uses ordinary `mod instruction;` discovery from the authored `main.rs` and
uses the existing `measure`/`run_once` start/stop/dump protocol and allocator
initialization. Existing parser, guest environment, libc dispatch policy, Rust
pin, and fixed build clock are imported unchanged from the driver checkout.

The standalone client uses mimalloc as its global allocator and the unchanged
repository `release` profile, including fat LTO and one codegen unit. This
measures actual product source linked into a public API client; it is not the
release CLI's complete link graph. The probe does not enable product micro
profiling or change product counters, fixtures, alignment, padding, or guest
environment variables.

Every standalone package starts from its source checkout's Cargo.lock.
`cargo metadata` may add the authored root package and prune unused packages;
before any build, the driver requires every resolved existing package's
name/version/source/checksum identity to occur unchanged in that source lock.
The resolved metadata and lockfiles are retained for review. Dependency source
paths select the actual base/head library; the authored Rust remains identical.

## Actions recipe

This temporary driver branch owns the probe under
`tools/benchmarks/probes/fresco-signal-dispatch/`. Its Pkl-generated
`.github/workflows/shipped-profile-measure.yml` runs the distinct Fresco signal
comparison and does not replace the product candidate branch. Dispatch with
exact 40-character base/head commit SHAs. Set SIGNAL_PROBE_BASE_SHA and
SIGNAL_PROBE_HEAD_SHA to those same commits when invoking the driver directly. The recipe uses the existing Ubuntu 24.04 / 32-vCPU lane,
Rust 1.100.0, Node 24, and Valgrind 3.22.0. It does not touch another workflow's
concurrency group or cancel other runs.

Equivalent driver command after preparing the runner:

```sh
SIGNAL_PROBE_BASE_SHA=<exact-40-character-base-sha> \
SIGNAL_PROBE_HEAD_SHA=<exact-40-character-head-sha> \
node tools/benchmarks/probes/fresco-signal-dispatch/driver.ts \
  /absolute/driver-checkout /absolute/base-checkout /absolute/head-checkout \
  /absolute/fresh-evidence-directory
```

Both expected SHA environment values must match the full lowercase hex format,
and the driver directly checks each source checkout's actual `git HEAD` against
its expected SHA before resolving dependencies or building.

The driver builds `base-a`, `head-a`, `base-b`, `head-b` using one fixed package
path and one fixed target path. It completely deletes the package tree and all
target contents before each build, and retains each actual ELF in its label's
directory before the next build. The authored source and marker file hashes
must remain identical in every build. Each independently built executable runs
three times through the same fixed guest path and the existing exact guest
environment. There is no product warmup. The driver rejects missing/extra
windows, nonzero termination dumps, changing oracles, nondeterministic
executions, unequal independently rebuilt A/A **ELF SHA-256 values**, or unequal
A/A instruction counts. It retains raw Callgrind data, symbolized actual ELFs, source hashes,
dependency identity, tool/environment identity, native-default results, symbol
sizes, and ELF section sizes, including on a failed comparison.

The final receipt reports each instruction delta and the exact summed ELF
symbol bytes containing `PREVIOUS_ACTIONS`. The strict gate fails any positive
instruction delta **or saved-action static-storage delta**. Two generations may
therefore expose an honest memory increase even if instruction rows pass.
Static symbol bytes are not a resident-memory measurement; full section output
is retained, and this receipt must not be described as whole-process RSS parity.

`measurement.json` is published before comparison failure. A red result is
evidence to investigate, not a reason to alter fixture contents, counters,
environment, thresholds, or oracles.
