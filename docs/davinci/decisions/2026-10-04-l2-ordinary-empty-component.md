# L2 ordinary empty-component provider

Issues: [#6838](https://github.com/ubugeeei-prod/vize/issues/6838),
[#6836](https://github.com/ubugeeei-prod/vize/issues/6836),
[#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

The preceding [original default-export receipt](./2026-10-03-original-default-export-receipt.md)
is the authentic source provider. The next bounded L2 family accepts one original
ordinary JS or TS non-definition module with one direct empty `ObjectExpression`
default export, original prologue directives and empty statements. Every other
statement, nonempty object, wrapper, import or declaration remains outside this
family. This is a semantic provider change; ordinary SFC product output and the
default compiler path remain unavailable in this change.

`ProgramReferenceSource` retains the original `AdmittedProgram` in place of its
raw Program pointer. No duplicate owner pointer is added. Its original
source/span/profile checks and sole declaration/reference walk remain intact.
Private resolution laws use original `parse_observed` admission; mutable invalid
span probes remain private resolver inputs and cannot replace admitted syntax.

A distinct private `ProgramOrigin::ordinary_empty_eligible` bit is initialized
from that authenticated observation: explicit module, default options, no JSX,
definition or unambiguous profile, no hashbang or original legacy literals, and
an actual sole direct-default parser receipt. Actual directives are allowed;
legacy escapes within them still refuse. Rejection is sticky in the existing
statement walk. The same existing export-default event joins the original
statement/declaration/default-keyword receipt to the actual direct empty object
and checked authored ranges. No additional parse, AST/source walk, stage,
serialization, Facts pointer, Box, annotation-storage reuse or span mirror is
introduced. Original setup eligibility, strict flags, ordinary export records,
observer order and interruption guards retain their independent semantics.

`OrdinaryEmptyScript` borrows only the same actual completed root ScriptUnit and
existing default Export row, requiring a sole unit, zero bindings/imports/script
references and unchanged strict eligibility. Producer preflight grants neither
File/template completeness nor descriptor membership. Completed File projection
requires actual File completeness. `VueOrdinaryEmpty` additionally borrows the
actual ordinary `ScriptView` and same original parser receipt, authenticating
full source and script-slice identity, JS/TS profile, default options, unit/span,
original body storage/length, root scope and uninterrupted complete File. Its
statement/default/object ranges remain borrowed projections; the clean parser's
unescaped export-token start supplies the fixed keyword range without scanning.
It grants no ordinary binding exposure, Options/Ref classification or native
product replacement. The selected native owner still checks descriptor custody;
its eventual product consumer must require the actual sole ordinary selection.

Vue File policy keeps the original staged `UnsupportedOptions` issue. Template
preflight may ignore only the exact unit/scope/statement issue authenticated by
the producer's completed-script proof, with no setup role present. Existing
complete-File finalization removes only that precise issue after the same
bounded family seals. Other diagnostics, invalid syntax, mixed setup and
interrupted template walks retain the issue and remain rejected. The private
selected-owner preflight similarly permits only that sole ordinary default
export and keeps calls, all other exports and reserved-name refusals intact.
Existing ordinary no-export behavior is unchanged.

Hosted laws cover genuine JS/TS directives, empty statements, comments, Unicode
and CRLF ranges; same original Program/source/comment identity through owner
moves; copied bytes, second Programs, foreign numeric units, alternate profiles
and parse options; wrong role, nested scope and extra units; nonempty options,
parentheses, TS wrappers/types, duplicate defaults and additional empty exports;
strict directive receipts and statement rejection before/after the candidate;
actual callback and template interruption before/after retained export rows;
and exact staged-issue ownership. Compile-fail docs retain construction,
cloning and live File/parser-owner borrow boundaries. A hosted layout law must
measure ScriptUnit at 48 bytes/alignment 8 and the admitted Program reference
carrier at 16 bytes/alignment 8 on 64-bit targets.

Source formatting, diff and unchanged 350-line source limits are checked without
a local build or installation. Fresh exact-head configured Actions must compile
and execute these laws and doctests. Protected full suites, all 100 unchanged
instruction ceilings and actual native Stack merge remain mandatory; source
implementation alone supplies no acceptance credit. No performance budget,
methodology, workflow, product fixture output or legacy/default path changes.

The first exact source Check 37133857957 compiled and passed its Rust laws,
layout checks and doctests. The tooling proof policy rejected one prefix-only
Options issue assertion. It now compares each entire authored default statement,
retaining the original issue count, unit/scope and complete File checks. Fresh
source Actions and protected acceptance remain required for this correction.

The corrected exact source `011af77756396e259da4c0e4b05125e913ea6218`
passes Check 37135183563. Protected candidate
`90c1546c29b34fbc0e0d4ff3741e934354812d38` passes Check 37136015932,
Musea 37136015567 and Nuxt 37136015542, including the full Rust/differential
suite, four Rust and tooling workers and all unchanged instruction ceilings.
PR #7613 actually merges as that verified signed commit at
2026-10-03T16:29:46Z, confirmed in fresh main. Linked L4 and product capture
acceptance remain separate unfinished dependencies.

TODOs remain the linked L4 default rewrite, real complete native module/map and
Vue runtime capture closure, broader Options API and ordinary binding exposure,
remaining JS/TS/Vue dialects and compiler fix-history/default migration. #6880
remains open. These dependencies must consume the sealed original provider
instead of inferring a bare object from source text.
