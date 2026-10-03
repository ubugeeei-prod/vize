# Native primitive const setup

Issues: #6838 and #6840. This additive slice follows native JS setup product
#7497 and retains its original NativeSfc, admitted Program and same-File
custody. Default compiler selection and #6880 remain unchanged.

The existing original Program walk now permits direct root Const declarations
with actual primitive literal initializers, beside the existing Let/Var family.
Original declaration kind, full-file source span, scope and initializer facts
remain authoritative. Foreign source/second Program owners and incomplete or
interrupted walks cannot construct VueSetup. Nonprimitive initializers, patterns,
imports, macros, ordinary/TS scripts and unavailable templates still refuse.
No source scan, additional parse, AST walk, stage or serialization is added.

Vue exposure admits the authentic Const binding class. L3 records the separate
SetupConst READ kind only for a genuine Const/PrimitiveLiteral declaration,
checking the same occurrence and File binding identity in its existing loop.
Only a retained direct identifier leaf with its one authenticated immutable
read becomes LiteralConstant. Calls, prototype/member reads, member writes,
compound expressions and mixed mutable reads retain FileDependent facts.
A read list alone cannot prove purity; general constant evaluation is unfinished.
Unsupported writes and unknown Const initializers never publish partial reads.

L4 reads the genuine class and keeps the same linked $setup access. Its setup
fragment copies the original SourceBlock once, then exposes immutable getters
without mutation setters. Existing Let/Var getter/setter code and eight whole
module/map fixtures remain exact. Generated-name and proxy collisions retain
their original typed refusals. Getter-only native exposure preserves the actual
lexical constant even when a consumer attempts Reflect.set on the returned state.

Five frozen complete source inputs cover numeric, boolean, null, BigInt and
string primitives, Unicode/comments, escaped binding names, declaration order
and mixed Let/Var updates. An isolated locked Cargo build generated their actual
native module/map objects from this source; its temporary example was removed.
The same custody law compares complete Recorded/NoLinks output and map objects,
checks actual declaration kinds and original body links, then retains the fresh
hosted capture. An independent actual Vue 3.5.35 compiler/runtime generated the
reference modules and ten real reference renders. Upstream may hoist literal
source outside setup; the native route keeps its original authored body intact.

The reference/native laws independently decode every mapping segment and check
all named links. Native execution invokes the actual generated setup exactly
once per input, verifies getter-only const exposure and failed writes, then runs
ten real Vue renders. Mutable updates keep the TEXT patch flag; primitive const
leaves omit it. Reference and native complete rendered trees agree. Native output
is never replaced by an upstream or legacy compiler result.

Focused current-source locked Cargo validation passes 41 lower/exposure/READ
laws and seven product setup laws; the combined original/new Node campaign has
31 passes without skips, including thirteen actual native setup calls and 26
native renders. The full-only hosted action adds the exact five-module law and
new runtime checks, retaining both const JSON outputs beside existing captures.
The actual merge-group tooling router invokes this action before its full
script suite. The previous #7497 candidate ran full Rust/instruction gates but
skipped the top-level schedule/manual-only tooling job, so it receives no new
hosted capture/runtime credit; historical campaigns remain source-bound. This
router wiring closes that current queue execution gap without adding PR work.
Ordinary tooling explicitly skips six const capture/native cases when the
fresh input is absent, and receives no native execution credit.

Fresh exact-head Actions, all unchanged instruction ceilings/ratchets, protected
native Stack candidate checks and actual merge remain necessary. Broader script,
macro, template-control, const expression evaluation, ordinary/TS output and
compiler-history/default replacement remain unfinished.
