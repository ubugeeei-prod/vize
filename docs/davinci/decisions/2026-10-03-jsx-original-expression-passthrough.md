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

## Evidence and hosted capture

Three new complete sources cover module/local scalar initializers, a normal
parameter-returning helper, a local binary initializer, an original function
call, a genuine module-scoped JSX read and helper-name collision. The pinned
Babel/Vue reference retains complete original transforms/maps and six real
runtime executions. Named native JSX reads must independently resolve to the
original and generated UTF-16 coordinates.

The 8 original-owning L3 laws and 2 scalar laws passed locally before the
team's disk constraint. The new local Rust whole-module capture failed during
compilation with ENOSPC before the test ran and receives no execution credit.
Local Cargo/rustc/npm installs and new probes remain stopped. Exact source
`34f45238` hosted Check run 37118594496, Rust job 111190833154, then produced
all three complete genuine code/map captures in failed-test stdout/JUnit.
Every manually authored original-expression role list agrees exactly. Only
the pending native code/map fields are populated from that capture; the
independent reference transforms, maps and execution fixtures stay unchanged.
The five pure Node reference/runtime/map/mutation laws pass with those genuine
modules. The original failed generation run stays intact, and fresh exact-head
hosted Rust fixture comparisons remain required; no assertion is weakened.

## Remaining work

Exact source `df1b478a` Check 37119522963 passed the complete hosted native
fixture comparisons. The original four-layer provider prefix actually merged
through the protected queue on 2026-10-03 at 12:23:43–44 UTC: #7486, #7503,
#7519 and #7538, ending at `1b9fce90998e`. Their exact queue Checks
37121082567, 37121083060, 37121084065 and 37121891628 passed, including the
unchanged instruction gate. GitHub rebased this remaining child to `2b05244a`
on that actual main and retargeted its base to main; all nine reviewed
production/test/reference/fixture payload files remain byte-identical.

Exact source `f49beb423c` Check 37122988988 and protected candidate
`134c4ad549ce` Check 37123717090 both completed successfully. Instruction
job 111204731926 verified all 100 benchmarks across three identical repeats,
all pinned ceilings and the unchanged ratchet. #7561 literally merged as
that candidate at 2026-10-03 12:59:16 UTC; fresh main retained all nine
reviewed production/test/reference/fixture payload files byte-for-byte.
TS erasure, compound JSX containers, wider attributes/slots/grammar and
legacy product routing remain unfinished.

## Scoped queue recovery

Appending this child to an already queued prefix produced a null-candidate
UNMERGEABLE entry with add/add conflicts against the squashed prospective
parent tree. The child must stay out of the queue until the real parent lands.
The public `dequeuePullRequest` mutation instead looked for its parent branch
queue and failed. Draft conversion did not clear the entry, and the native
Stack refused a temporary base-main edit while the child remained stacked.

Closing and immediately reopening only draft #7561 cleared its queue entry.
The before/after GraphQL observations retained Stack #7504's five ordered
entries, all five source heads and the four healthy queue candidate identities.
No healthy prefix was dequeued or unstacked; no source or branch was deleted.
After the actual prefix merge, inspect and adopt GitHub's rebased child head
before any guarded publication. This is an observed metadata recovery for
this native Stack API limitation, never a substitute for fresh required checks.
