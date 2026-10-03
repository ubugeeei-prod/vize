# Original non-JSX expression passthrough

Tracked in #6838, #6839, #6840 and #6829 after native JavaScript JSX modules
in Stack #7504. This is a bounded source admission change, not roadmap closure
or product replacement.

## Decision

Ordinary scalar initializers and helper-function returns previously made the
entire L3 JSX view refuse, even though the original L2 File was complete.
The existing same-walk preorder records now expose a readonly owner-bound
subtree iterator. L3 gives `OriginalExpression` only to a whole original root
whose existing subtree contains exclusively genuine expression records.
A JSX descendant, element, fragment or container prevents that role.

All rows remain in their original order beside the same retained parser
observation and completed File. No source-string classification, independent
AST, new parse, AST visitor, metadata table or pipeline stage is introduced.
L4 replaces the genuine admitted JSX roots and copies all other original
module bytes through its existing writer. Original expression semantics and
scope remain authored; helper imports still use actual all-scope File
collision facts. This role grants no JSX compound-expression interpretation.

The previous ordinary-expression refusal law becomes an exact source-role
and retained-owner law. Compound JSX expressions, dynamic attributes,
fragments, spreads and incomplete lower owners keep their refusals. The
foreign-owner query, moved owner, source-window and TS-erasure gates remain.

## Evidence and outstanding capture

Three new complete sources cover module/local scalar initializers, a normal
parameter-returning helper, a local binary initializer, an original function
call, a genuine module-scoped JSX read and helper-name collision. The pinned
Babel/Vue reference retains complete original transforms/maps and six real
runtime executions. Named native JSX reads must independently resolve to the
original and generated UTF-16 coordinates.

The 8 original-owning L3 laws and 2 scalar laws passed locally before the
team's disk constraint. The new Rust whole-module capture failed during
compilation with ENOSPC before the test ran. No capture or execution success
is claimed from that failure. Local Cargo/rustc/npm installs and new probes
are stopped; exact-head hosted Actions must produce the genuine native
code/map captures and validate their strict fixture comparisons. Pending
native fixture fields and failed-test stdout/JUnit are explicit draft work;
assertions and source policy are not weakened to accommodate generation.

## Remaining work

Whole-module native/reference fixture agreement and fresh exact-head Actions
remain required before this draft becomes ready. The protected merge queue
must retain all full-suite and instruction gates, with actual provider
ancestry and actual merge tracked separately. TS erasure, compound JSX
containers, wider JSX attributes/slots/grammar and legacy product routing
remain unfinished.
