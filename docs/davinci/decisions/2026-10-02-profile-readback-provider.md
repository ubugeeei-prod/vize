# Profiler metric readback provider

Tracked in [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).

L0's profiler owns collection and its existing metric identities. Its two
existing span/counter readbacks become public so the host can consume those
owned records directly. Each body, allocation-tracking pause, shard iteration,
metric clone and plain-before-attributed ordering is retained. This adds no
snapshot wrapper, pipeline stage or serialization between levels.

The real dependent consumer is the existing machine-readable JSON export,
which moves to Carton in the child change. The provider alone leaves that
export and all callers in their existing owner. Collection, clocks, allocation
counters, global state and the disabled relaxed-atomic branch are unchanged.

The controlled Rust laws retain separate plain/attributed buckets, counter
aggregates and ownership after the collector is cleared and dropped. The
TypeScript replay accepts only complete old or complete public signatures,
rejects partial/duplicate state before writes, and is idempotent.

The full profiler/platform isolation and effective framework configuration
remain unfinished. Actions and the protected merge queue must validate the
actual source head and all 100 unchanged instruction ceilings before this
bounded provider/host Stack can merge.
