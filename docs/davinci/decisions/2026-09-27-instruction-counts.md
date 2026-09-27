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

The first Linux bootstrap [run 36303887819](https://github.com/ubugeeei-prod/vize/actions/runs/36303887819)
passed default/feature harness tests and collected 87 probes, but the two
constant-folded observer routines produced zero-count dumps: start and stop
remained in the same already-translated basic block. The measured routine
therefore crosses a non-inlined call boundary after instrumentation starts.
This ensures even a short routine enters an instrumented block. The fixed
benchmark wrapper overhead is included in every candidate and ceiling; it
does not add a production call or stage. Zero-count dumps remain rejected.
`window_protocol = "callgrind-client-call-boundary-v1"` fixes that protocol
before baseline pinning. The raw ceiling includes each probe's optimized
call/return and `black_box` wrapper instructions, including its result ABI;
it is not a count of product work alone. A product observer can compile to
zero work while its raw benchmark still counts the wrapper. No overhead is
subtracted, and no product work is added to make a measurement positive.

The reference build is Rust 1.98.0, Linux x86_64, generic `x86-64` target CPU,
the `ci-opt` profile (thin LTO, 16 codegen units), and the existing counting
allocator over mimalloc. Valgrind is Ubuntu 24.04's 3.22.0. Cache and branch
simulation are disabled. Reports name the exact Rust/Valgrind/glibc versions,
source commit, workflow run, fixture digest and window kind. CPU, package and
kernel details are retained in a separate environment artifact for diagnosis.

The next Linux [run 36304195926](https://github.com/ubugeeei-prod/vize/actions/runs/36304195926)
completed all 100 probes in all three executions, with 98 identical. The
stress-interpolation analysis and large Vapor compilation differed by 457
instructions. Comparing exclusive per-function costs attributed the complete
delta to mimalloc's lazy page-map submap allocation and its OS allocation
calls; compiler-stage costs matched. Randomized allocator address placement
can move this initialization between probes. The validator rejected the run
and emitted no candidate budgets.

The allocator protocol is therefore explicitly
`counting-mimalloc-reserve128m-preinit64m-purgeoff-v1`: reserve 128 MiB at
process startup, allocate and drop a 64 MiB byte-buffer capacity in harness
initialization before any measurement, and disable time-based purging. The
large allocator-only allocation registers its address range with the lazy
page map. No product routine is executed as warmup. The counting allocator
and measured stage work are unchanged. The driver clears inherited mimalloc
options, requests verbose runtime diagnostics, and requires the allocator's
actual 131072 KiB reservation and option values plus the completed harness
preinitialization marker in each process's raw log. Failure of that setup
fails collection. Identical counts in three executions remain mandatory;
these settings alone are not proof of stability. The pinned allocator
source is [libmimalloc-sys 0.1.49](https://docs.rs/crate/libmimalloc-sys/0.1.49/source/c_src/mimalloc/v3/src/),
specifically `init.c`, `arena.c`, and `page-map.c`.

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

The [reference run 36304969684](https://github.com/ubugeeei-prod/vize/actions/runs/36304969684)
verified all 100 probes identically in three executions with the fixed
allocator protocol. Independent artifact inspection reconciled all 300
named dumps, 36 zero-count termination dumps, and 36 successful allocator
setups with the report and candidate registry. Raw logs report mimalloc
v3.3.2 and the actual 128 MiB reservation. The measured TOML is pinned
without a margin: source `f5d4f29b72bb21b9e567477149ee40cbe0bdb9f7`, artifact
`10926957194`, artifact digest
`23364ef28ee1125be1f5a1e141835a3574014ea7eceb315511c4460800d34906`, and TOML
SHA-256 `b86442507bc16b5f2acf260d291c1de39c911e735d5bb1d2b7db891fa1698607`.
The source is the tested PR merge commit, whose second parent is branch head
`5ac03e84188ff9b329398bd6ff88b2ef1fbbd4f3`. Later runs measure their current
source against this historical measured reference; provenance does not
require candidate and baseline source hashes to match.

The required `Check` workflow now calls the instruction workflow and includes
its result in `test-report`. Every call validates the complete pinned
registry and ratchets against the exact fetched event base commit. If the
base has no registry during initial introduction, only the above exact
reviewed TOML hash is accepted. Missing or empty registries always fail.
Every merge-group call performs the three-run collection and unconditional
ceiling comparison. Unrelated PR calls check the registry and ratchet without
compiling the benchmark suites; the path-selected standalone PR trigger
measures gate changes before queue entry. Reusable workflows retain the
caller workflow name, as specified by the
[GitHub reusable-workflow context](https://docs.github.com/en/actions/reference/workflows-and-actions/reusing-workflow-configurations#github-context),
which distinguishes that standalone measurement from
the fast `Check` PR call. Queue measurement is selected directly by the
merge-group event and cannot take the PR path. Required queue verification
is still unfinished until that exact final revision runs in the queue.

Cross-job verification [run 36305573043](https://github.com/ubugeeei-prod/vize/actions/runs/36305573043)
then rejected thirteen ceiling comparisons, despite all three executions
within the new job agreeing. The compiler source, base commit, physical
runner, CPU model, and glibc version were unchanged. Exclusive function
comparison attributed the complete differences to `__memcmp_avx2_movbe`
and `__memcpy_avx_unaligned_erms`, whose paths depend on input addresses and
page boundaries across builds. For example, DOM code generation differed
by 737 instructions entirely within the library comparison routine. The
above draft registry is therefore an unaccepted bootstrap reference, not
a proven queue baseline. No ceiling was increased to hide this result.

Before final pinning, `libc_dispatch` records a fixed `GLIBC_TUNABLES` hardware
capability mask disabling AVX, AVX2, AVX512, ERMS, FSRM, SSSE3, and SSE4 paths
and the corresponding glibc preferred-dispatch flags. This keeps the
baseline SSE2 library implementation. The dump parser rejects actual
hardware-specific libc routine names, so requesting an environment variable
without changing dispatch cannot publish a baseline. Per-suite binary
digests are diagnostic artifact provenance; they do not require future
candidate binaries to equal historical binaries. The libc mask follows the
[GNU C Library hardware-capability tunables](https://sourceware.org/glibc/manual/latest/html_node/Hardware-Capability-Tunables.html)
and its [2.39 comparison selector](https://raw.githubusercontent.com/bminor/glibc/release/2.39/master/sysdeps/x86_64/multiarch/ifunc-memcmp.h).
Final pinning now requires matching counts in at least two separate Actions
job builds with the same compiler source, in addition to three identical
executions within each job. The draft remains blocked until that proof and
the exact queue enforcement pass.

`fixture_digest = "input-identity-v2"` preserves input identity across
structural moves. Direct fixture files hash their exact bytes independently
of their paths. The four level storage probes hash their exact template
constants or included dashboard bytes, independently of the benchmark
target, function, module and type names. Other synthetic probes retain a
generator-source fingerprint and included fixture bytes, with include paths
normalized. The driver accepts exactly one side of the bijective
`davinci_storage` to `l1_to_l2_storage` target/source move, preserving all
benchmark ids, windows and numeric ceilings. A fixture label or path may
change only while its input digest remains identical; changed input bytes
still fail. Counter/span/timing identity renames are a separate registry and
cannot reset these instruction budgets.
