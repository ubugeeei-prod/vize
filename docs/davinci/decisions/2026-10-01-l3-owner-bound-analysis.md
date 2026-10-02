# Owner-bound native L3 analysis (2026-10-01)

Tracked in [#6839](https://github.com/ubugeeei-prod/vize/issues/6839), after
the bounded canonical L2/native decision provider. L4's first real SSR
consumer exposed that bare mutable `DecisionTables` could be paired with
another artifact whose local node indices happened to match.

`vize_l3::decision::NativeAnalysis<'owner, 'arena>` retains the actual
`&'owner vize_l2::artifact::Artifact<'arena>` and private complete tables.
Its `artifact()`, `policy()` and `tables()` accessors are read-only. It has
no public constructor, mutable table accessor or consuming table extraction.
Cloning a table creates detached scratch and cannot claim completed analysis
for any owner. The artifact borrow prevents unsealing it while decisions live.
L4 accepts this sole bound result and derives its owner and selected policy.

The sole native producer moves byte-exactly to L3 in a move-only commit;
`tools/commands/davinci/move-native-decision-producer.ts` replays the rename
on a fresh parent. A following commit wraps the actual producer result.
L3 takes the normal downward L2 dependency needed for this ownership boundary;
the conversion crate only re-exports the native producer and still owns the
existing flat-program lowering. There is no normal/build legacy dependency.

The existing single event walk supplies every node id. Each node/control
insertion checks the sealed owner's key range and rejects replacement of an
existing row. Finishing checks open frames and exact node cardinality only;
bounded unique keys plus cardinality prove complete coverage without another
walk, expression parse, numbering, serialization or pipeline stage. Errors
discard the private scratch builder; no partial analysis escapes.

The neutral/output policy rules, authored binding order and control
containment remain unchanged. Placement stays `Inline`. Slot grouping,
hoist/cache decisions, complete target criteria, demand-only flat programs,
product selection and #6839 completion remain unfinished. The existing 100
pinned instruction rows execute zero calls of this producer; unchanged
ceilings do not demonstrate native adoption or performance acceptance. A
real native workload waits for the fused checked L1-to-L2 producer.

Six native integration laws retain existing family/policy/control coverage
and prove exact owner retention for two artifacts with equal local ids.
Three private builder laws reject out-of-range/duplicate node and control
keys and incomplete cardinality. Three compile-fail API laws reject arbitrary
sealing, table mutation and owner unsealing while borrowed. Two existing
policy laws remain unchanged. Selected real source modules compile against
cached L0/OXC without stubs or dependency rebuilds; full current-workspace
integration, Clippy, legacy differential output and unchanged instruction
acceptance remain Actions requirements before publication/queueing.
