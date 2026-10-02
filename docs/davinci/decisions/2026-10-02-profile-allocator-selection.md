# Host profiling allocator selection

Tracked in [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).
This follows the [profile export boundary](./2026-10-02-profile-export-host-owner.md).

L0's existing `ProfilingAllocator<A>` already wraps a caller-selected
`GlobalAlloc`. Its only concrete allocator dependency is the `System` default
type parameter and the `new`/`Default` convenience implementations. Remove
those conveniences from L0. The existing `std::alloc` re-exports retain the
identical `core` `GlobalAlloc`/`Layout` contracts; the crate remains std-bound.

Carton's `profile_allocator::system_allocator()` selects `std::alloc::System`
and returns the exact L0 wrapper through its existing const `from_allocator`.
The real SFC allocation-budget binary and Carton profile-export/schema binary
select this host constructor. Their counter windows, fixture bytes, allocation
ceilings, nested-span assertions and serialized schema remain unchanged.
The CLI and SFC attribution executable still supply their existing mimalloc
allocator directly. No normal/build dependency, target/optional edge, lock entry,
pipeline stage, metric conversion or serialization is added.

Every allocator hook, delegation argument, failure/success branch, counter,
suppression operation and disabled relaxed-atomic load remains byte-identical.
The returned generic type retains the same layout and ownership; there is no
second allocator wrapper or dynamically dispatched backend.

This is an independent allocator-selection boundary using an existing provider.
No whole source file moves. TypeScript replay preflights the exact old and new
allocator source, both actual host call sites and the Carton module declaration
before writing. Missing/partial/colliding or changed input rejects with zero
writes, and repeating integration preserves every byte.

The existing SFC preferred-L0 source gate admits only the exact System factory
initializer in its allocation-budget binary. It still rejects Carton storage,
alternate allocator types, additional calls and the same host call in compiler
source. Replay validates and updates this executable gate before any write.

Finite source proof compares the original allocator hooks and actual callers
with signed main. The public host doctest verifies const construction and exact
L0 return-type identity. Actual current-source Actions must execute both
allocation/export binaries, compile the unchanged mimalloc consumers, validate
the dependency/storage inventories and pass all 100 unchanged instruction
ceilings before protected-queue completion.

L0 still owns thread-local allocation counters and suppression, `Instant`, the
profile stack, global collection, `RwLock` shards and native arena/recursion
facilities. This change does not make the crate no-std or finish #6834; complete
thread/clock/platform isolation and effective framework model placement remain
unfinished.
