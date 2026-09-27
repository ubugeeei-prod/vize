# Per-stage instruction ceilings (#6868)

Instruction measurements reuse the committed benchmark routines and
`StageWindow::measure` boundaries. The `instruction-counts` feature is
confined to the benchmark harness, with an optional Linux x86_64 dependency
on `valgrind-requests`. It adds no production pass, pipeline stage,
serialization, or normal dependency to a level crate.

New workflow and test filenames use `level-instruction-counts`. The harness,
benchmark targets, fixture paths, and registered ids retain their existing
names because this gate measures the pre-migration routines and must reconcile
with their normative allocation registry. Those references do not introduce
new product crate, module, type, or serialized benchmark identities. Their
eventual renames belong to the ordered level migration, with measurements
preserved across that move.

The feature bypasses Criterion sampling. Each routine runs once per fresh
suite process. Callgrind starts with instrumentation off. The harness starts
instrumentation immediately before the routine and stops immediately after
its return, before dropping its returned value, matching the allocation
probe boundaries. Stage setup and teardown stay outside `StageWindow`.
Panics stop collection and fail the suite; they publish no completed dump.
Named client dumps reset their counters. The reconciler rejects duplicate,
missing and unregistered dumps, nonzero process-termination totals, invalid
numbers, and any event set other than `Ir`.

The reference build is Rust 1.98.0, Linux x86_64, generic `x86-64` target CPU,
the `ci-opt` profile (thin LTO, 16 codegen units), and the existing counting
allocator over mimalloc. Valgrind is Ubuntu 24.04's 3.22.0. Cache and branch
simulation are disabled. Reports name the exact Rust/Valgrind/glibc versions,
source commit, workflow run, fixture digest and window kind. CPU, package and
kernel details are retained in a separate environment artifact for diagnosis.

The initial measurement must include all currently registered allocation
benchmarks. The issue's original 102-row count was a snapshot; the current
registry has 100 rows. Registry reconciliation owns the count and catches
future additions. Existing legacy probes are benchmark oracles, not a
legacy-backed product path. Future native L1/L2/L3/L4 probes must be added to
the same registry as those stages become available. In particular, the L3
hoist move must compare against an isolated existing DOM hoist analysis
probe before that migration is accepted; the whole-transform probe alone
cannot establish that narrower claim.

`instruction-budgets.toml` is a separate registry because `budgets.toml` has
a strict existing four-field row schema and a source-length ceiling. The
measurement JSON has its own schema. No zero or guessed instruction budget
is accepted. A baseline is emitted only after all three executions return
identical identities and counts. Every pinned ceiling is checked with no
percentage tolerance, and every existing ceiling may only decrease. Dropped
ceilings and methodology drift fail. Fixture/window changes require a
reviewed measured baseline while preserving the existing numeric ratchet.

Bootstrap sequence:

1. Keep the implementation PR in draft. Its path-selected workflow collects
   three executions and uploads raw named dumps, logs, the independent JSON
   measurement, and the measured TOML candidate.
2. Read that Actions artifact. If any count differs, diagnose the unstable
   routine/environment before pinning it. Do not replace equality with a
   generous ceiling. Counted instructions can vary when allocator paths or
   CPU-dispatched library routines vary, so stability is a measured property.
3. Commit the artifact's measured TOML as
   `docs/davinci/plan/instruction-budgets.toml`, connect `merge_group`, and
   make the enforcement step unconditional for merge-queue runs in the same
   PR. Missing baselines must fail there. Run the exact revised PR in Actions
   and verify the queue's required check before marking #6868 complete.

The measurement command is:

```sh
node tools/benchmarks/scripts/instruction-counts.mjs --collect --out "$RUNNER_TEMP/instruction-counts"
```

The gate and ratchet command is:

```sh
node tools/benchmarks/scripts/instruction-counts.mjs --check \
  --measurement "$RUNNER_TEMP/instruction-counts/measurement.json" \
  --base-budgets "$RUNNER_TEMP/base-instruction-budgets.toml"
```

Client windows and one-shot collection follow the primary
[Valgrind Callgrind manual](https://valgrind.org/docs/manual/cl-manual.html#cl-manual.clientrequests).
The parser reads exclusive `totals: Ir` from the
[Callgrind file format](https://valgrind.org/docs/manual/cl-format.html);
it never sums inclusive call-edge costs.
[valgrind-requests](https://docs.rs/valgrind-requests/1.2.0/valgrind_requests/)
provides the client requests without introducing another benchmark runner.

Unfinished until an actual Actions baseline and strict required merge-queue
enforcement are verified. Wall-clock and resource budgets stay nightly.
