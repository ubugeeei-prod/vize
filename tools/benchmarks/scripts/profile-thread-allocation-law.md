# Allocation scopes and threads

Per-span allocation counters use monotone thread-local requested-call/byte totals. A guard samples that thread at entry and drop; recorded nested children are deducted for self values. Other threads and descendant backend processes are outside that span. Process-wide cumulative allocator totals remain a separate measurement.

Profiling builds make ProfileGuard non-Send with a zero-sized marker. Normal builds retain the existing guard definition and constructor after cfg elimination. The marker adds no clock, heap allocation or runtime synchronization. A negative compile test protects the profiling contract, and the normal carrier preserves the existing Send contract.

The forced allocation carrier opens two worker scopes simultaneously. A requests 1024 bytes with a 256-byte child; B requests 4096 bytes with a 128-byte child. While both are open, the caller requests 8192 bytes under its own scope. Exact inclusive/self assertions reject global-delta attribution and cross-thread child nesting. Atomic handshakes keep setup/teardown outside the window; the allocator is real.

Self means inclusive minus recorded same-thread child spans. Uninstrumented callees remain in self. Durations are wall time, including waits, rather than CPU self time. Inclusive rows overlap and must not be summed as a process allocation total.

A non-Send guard does not authorize holding a scope across arbitrary async suspension. A local executor can poll unrelated futures on the same thread. The current four Patina probe await sites run through synchronous probe_types_at_offsets and pinned corsa_runtime 2.0.0-beta.2 block_on: its executor owns and polls one future on the caller thread and parks; spawn creates a separate OS thread. These sites describe caller-thread work, excluding worker/backend allocation. This is a qualification of that current path, not general async allocation support.

Published source was read in RAM and matched Cargo.lock checksums: corsa f5f74730a7e068220241d94d84530932648a46ccfa6e053867f325e74ba155ba and corsa_runtime 13cac6caa1fee9c9e0bb06ed949b0b53b3d2fb24ea9c0e65bdd987134693b7df, VCS ea08d249a089b510486fb963bf4b0157938eecbe. The relevant runtime source is src/core/corsa_runtime/src/executor.rs and task.rs.

The dedicated Actions job runs the small normal/instrumented carriers, the compile-fail guard law, existing nested export and observer exclusion laws, then compiles the profiling CLI. It does not start a project throughput campaign.
