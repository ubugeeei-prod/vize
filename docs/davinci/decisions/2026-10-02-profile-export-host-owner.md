# Profiler JSON export host ownership

Tracked in [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).
Depends on the [existing readback provider](./2026-10-02-profile-readback-provider.md).

The existing JSON exporter and its six wire laws move unchanged in a dedicated
move-only commit to Carton. Integration replaces the foreign inherent method
with `profile_export::export_report(&Profiler, &ProfileExportOptions)`, consuming
the same two owned readbacks directly. The real CLI writer, Curator browser
timing export and SFC attribution benchmark call that host function.

Carton owns schema version, serialized field order, deterministic ranking,
entry caps, truncation accounting, duration saturation and pretty JSON with a
trailing newline. Every field, formatter and ordering rule is retained. The
tool version still comes from the identical workspace package version. There
is no additional snapshot, serialization stage or metric conversion.

L0 retains metric identities and the collector, timers, global state,
allocation counters, allocation-tracking pauses and disabled relaxed-atomic
branch. It has no host export re-export or normal/build dependency on Carton.
The existing L0 observer/dump and L1-to-L2 provenance wire tests use permitted
dev-only host oracles. Curator declares its real normal host dependency; the
SFC example uses a dev dependency. Optional/target/transitive normal and build
dependency direction remains enforced.

The host import gate admits only the exact JSON symbols at the three reviewed
production callers. Carton storage, wildcard/group imports, other symbols and
level source paths still fail. Windows path separators retain that policy.

TypeScript replay validates complete source/target ownership, exact reviewed
readback/export/test bytes, actual caller signatures, executable host gate
wiring, semantic TOML declarations and lock edges before writes. The byte guard
accepts only the precise pure input, precise replay output and canonical Rustfmt
output; whitespace inside literals and golden JSON remains significant. Partial
moves, collisions, malformed or aliased/target dependencies and stale callers
reject without changing any file. Repeating every phase preserves the integrated
bytes.

Cached actual-source Rust laws verify byte-exact JSON, separate attribution,
caps/tie-breaking, real nested allocation counts and the committed schema.
Actions must still compile the actual CLI/browser/example/oracle consumers;
exact-head full checks and all 100 unchanged instruction ceilings precede the
native Stack's protected merge queue.

This is a bounded exporter boundary. Full profiler portability, clock/global
and platform isolation, and effective framework model placement remain open.
