Source operation profiles are diagnostic builds. Compare product wall time, RSS,
artifact size and hardware counters using normal binaries with identical source,
flags, inputs, backend and thread limits. The instrumented binary deliberately
adds work; its elapsed time is not a product-performance receipt.

```sh
cargo build --locked --profile ci-opt -p vize --features profiling
target/ci-opt/vize check --profile-json /tmp/profile.json path/to/project
```

`profiling` installs the allocation wrapper and activates `micro-profiling` in
the shared foundation/host. `micro_count!("operation.bytes", value)` records one
invocation and its explicit units. `micro_span!("function", expression)` records
invocations and joins synchronous inclusive/self timing and allocation totals
through its `key`. File, module and line identify the actual Rust callsite.
Ordinary builds erase both macros and never evaluate a counter's operand.

| Receipt                 | Meaning                                                                                                            |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `micro[].calls`         | Calls to that instrumented source site across Rust threads in this process                                         |
| `micro[].units`         | Sum of the explicitly supplied unit; the operation name documents bytes versus calls                               |
| `spans[].wall_ns`       | Inclusive/self wall time on a synchronous thread; overlapping worker durations are not elapsed process wall time   |
| `spans[].alloc`         | Allocation-like calls/requested bytes on that span's thread; worker allocations are separate                       |
| `allocation.operations` | Successful/failing alloc/zeroed/realloc/dealloc API counts and requested/released layouts across this Rust process |

Descendant TypeScript backends are outside Rust counters. These allocation sizes
are cumulative traffic, not peak live heap or RSS. `realloc_old_bytes` and
`realloc_new_bytes` are layouts, not physical bytes copied by an allocator.
Deallocations can release storage created before the profile window.

Current sites distinguish complete one-shot XXH3 calls from streaming writes
and digest queries. Multiple `finish`/digest queries do not imply multiple
complete input hashes. Streaming constructors remain `const` and are not
observed. UTF-8 validation, host source reads and shared canonicalization have
their own sites. Direct filesystem/parser/hash calls outside these sites are
unobserved; absence of a counter does not establish zero work.

Counters use 32 cache-line-separated slots selected by a thread-local ordinal,
with exact atomic aggregation even when more than 32 threads share slots.
First registration takes one lock and may allocate; later counter writes use
TLS and two relaxed saturating atomic updates. Timed spans additionally read
clocks and update the existing sharded span store. Collector storage/readback
allocations are suppressed. No instrumentation-owned hasher is wrapped, and
allocation hooks never invoke source counters.

Stop producers before snapshot/reset: concurrent snapshots are not coherent
cuts. Source counters are untruncated; timing entries retain the ordinary
budget and explicit dropped-span accounting. Owned-snapshot exports do not
read the global source collector. The Actions `Source operation profile` job
tests the normal disabled law, collector exclusion, nested attribution and
64-worker aggregation, then preserves three rounds of normal, instrumented
runtime-disabled and instrumented-enabled counter overhead with checksums.

Hardware `perf stat`, software `perf record` and Callgrind instruction counts
are separate measurements. A PMU capability failure is unavailable evidence,
never a zero instruction count. The existing instruction-count workflow keeps
Callgrind `Ir` as a simulation metric.

Selected Canon editor identity constructors use `micro_default_hasher!`: normal
builds expand directly to `DefaultHasher::new()`. Instrumented builds count
builders, actual byte-write operations/bytes (including separators), and finish
queries at each original callsite. This adapter is deliberately restricted to
Rust 1.99's known DefaultHasher behavior; exact digest tests cover string domain
separators, primitives, tuples, paths, empty writes and repeated finish queries.
It is not a replacement for FxHashMap or arbitrary custom hashers.

Croquis counts setup/plain parse requests and setup semantic-analysis requests
separately from actual OXC attempts. Initial and recovery parses have separate
source spans and byte totals: one recovery request may issue multiple parses.
The native standalone control executes real valid setup/plain inputs and an
incomplete-member repair, checking exact request/attempt totals and input bytes.
