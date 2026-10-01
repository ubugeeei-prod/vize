# Authored markup allocation windows

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The authored-markup allocation test runs as an explicit Cargo integration
target with `harness = false`. Its main function executes the existing
assertion body directly. Every cold parse, arena comparison, markup rule,
and allocation ceiling remains unchanged. This follows L0's existing
`remark_zero_cost` standalone integration-test contract.

`CountingAllocator` measures all threads in the process. A single libtest
test still runs on a worker while the runner waits on its result channel.
The runner can allocate the first four `std::sync::mpmc::waker::Entry`
slots during the first parse: four 24-byte entries add 96 bytes and one
allocation to a window whose parser otherwise allocates one 16,368-byte
arena chunk. Process main execution removes that concurrent runner;
allocations from work genuinely spawned by the measured routine remain
visible. Neither thread-local counters nor warm-up parses replace this
measurement contract.

The [original source-head run](https://github.com/ubugeeei-prod/vize/actions/runs/36926475360)
failed on ordinary `parse`, not on `parse_with_authored`: the first
Gallery parse recorded two calls / 16,464 bytes, and the second recorded
one call / 16,368 bytes. The parse, surface builder, fidelity verifier,
arena, harness and budget-test sources match the preceding Program layer
and main `a9eecd42569473a382cdce5cceecc7022fa9c42d`.

An isolated diagnostic built the complete public L1 source at
`57ac527ae0d0315c18f3fb5c0394676a382c63d9` against matching cached Rust
dependencies. All four ordinary/authored pairs used one arena chunk and
equal arena storage. A controlled foreign-thread channel wait added the
same 96-byte Waker allocation; a raw allocation stack identified its
actual `Vec<Entry>` growth. macOS also allocates 64 bytes for the fresh
channel's pthread mutex; Linux uses an inline futex mutex. This scoped
diagnostic establishes the measurement contamination, not full Linux
workspace acceptance.

Cargo's full workspace test command continues to execute this target.
The static test inventory recognizes its explicit Cargo registration and
actual main entrypoint as one case; ordinary tests retain their scanner,
while benchmarks and unregistered helper mains receive no test credit.
The report action sets up the existing pinned Vite+ action and Node runtime,
then installs only the root package's locked dependencies with lifecycle
scripts disabled. Cargo target accounting uses the existing `@iarna/toml`
2.2.5 dependency; the report must install it before collection. This fixes
the fresh-runner module failure observed in the required report job on
`59fa68836210274f5f906fb191addc9348ee33e1`, while preserving collection,
artifact upload and the final required-job aggregate.
The [hosted report on `d95bf12b8`](https://github.com/ubugeeei-prod/vize/actions/runs/36937207450/job/110622282693)
successfully installed the pinned parser, collected the inventory and
uploaded the artifact. Its final aggregate rejected the tooling failure
caused by the older law's fixed collector and uploader step positions.
The workflow law locates these stages by their names and checks their order:
pinned setup, locked root install, collection, then upload. Each stage must
run unconditionally and propagate failures; the same-job final aggregate
and instruction-check failure, cancellation and skip rejection remain required.
Fresh exact-head Actions and protected merge-queue checks remain required.
No product route or native feature gains completion credit from this
harness correction.
