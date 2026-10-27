# Source CPU evidence

`source-operation-perf.ts` profiles an already prepared executable. Fixture generation,
binary hashing, the Node driver and report rendering stay outside perf's measured child.
The counters include the executable's threads and inherited descendants, including a
checker backend. They are not Rust-only instruction counts. Callgrind `Ir` is a separate
simulation metric.

Hardware collection and admission are separate. Raw counter values and stderr remain
available when admission fails. Missing events remain null. Positive instructions with
zero cycles fail sanity. Retired instructions and branches use a separate `core` cohort
from the `cache` cohort. CSV values are already scaled by perf; scheduling below 99.9%
is labeled a scaled estimate and does not qualify for strict comparison.

The runner compiles the original Rust admission controls outside all measured windows.
`--binary` must name a new output path; an existing executable is rejected. The build
receipt binds the source hash, compiler arguments/version and actual binary bytes:

```sh
taskset -c 0-7 node tools/benchmarks/scripts/source-pmu-calibration.ts \
  --binary /tmp/pmu-calibration --perf /usr/bin/perf --out /tmp/pmu-admission
```

Use the same actual CPU affinity for calibration and product measurement. Each control
runs in a fresh process. Instruction and branch workloads must produce a larger signal
than the baseline, and dependent loads over a larger working set must produce an observable
cache-event signal after subtracting a separate setup-only control for each working-set
size. Table initialization alone cannot qualify the cache cohort. The raw CPU,
microcode, kernel, PMU type, affinity, perf version,
source hash and executable hashes accompany the result. Calibration is scoped to that
environment; it is not a Vize performance result.

CPU source profiles use the software `cpu-clock:u` event at 99 Hz and retain the native
DWARF stack trace. Recording skips its build-ID post-pass. Exact executable paths and
bytes must remain available for the immediate report and later replay. SHA-256 before
and after the report detects a replaced executable. The Linux timeout supervisor bounds
recording and report rendering separately. A positive sample count, resolved source rows,
and successful error-free symbolization are required for source-profile admission.
Kernel-symbol warnings do not discard a successful user-space trace.

For macOS, establish the `time -l` scope before comparing paths:

```sh
node tools/benchmarks/scripts/source-macos-rusage-scope.ts \
  --rustc /absolute/path/to/rustc --out /tmp/mac-instruction-scope
```

The original carrier compares the same deterministic work in one native process,
two worker threads, and a waited native child. It saves the exact compiler invocation,
source, binary identity, complete time output, worker checksums, child PID and exit.
On the measured Apple M2 Max host, three fresh-process rounds showed worker-inclusive
counts and exclusion of the waited Rust child: baseline-adjusted worker/single ratio
2.0101 and child/single ratio 0.0362. The measured carrier source SHA-256 was
`100401287ac8f60148284e928f7ae0b03724abd510ec69a6bb56280121a3148e`.
This observation does not validate Node or Go process-tree totals. Keep those paths
unqualified until their own scope controls run.
