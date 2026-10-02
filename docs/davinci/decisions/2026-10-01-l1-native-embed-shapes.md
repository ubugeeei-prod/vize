# L1 native handler bodies and slot parameters

Tracked in [#6836](https://github.com/ubugeeei-prod/vize/issues/6836), following
the [typed-embed design](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929).
This genuine dependent slice extends the
[bounded retained syntax provider](./2026-10-01-l1-native-embed-syntax.md).

`parse_once` accepts caller-selected JS/TS `HandlerBody` and `SlotParams` in
addition to Expr and Program. Each uses one private OXC Program parse in L0's
shared arena. HandlerBody is parsed inside `()=>{\n` and `\n}`; SlotParams is
parsed inside `(\n` and `\n)=>{}`. Newlines keep authored trailing line comments
from consuming generated delimiters. The existing 31-unit admission and OXC
safety guard both inspect the complete wrapped input before parsing. Larger
valid snippets remain `TokenBudget` holes; full grammar admission is unfinished.

HandlerBody exposes only authored directives and statements. Return is legal
in this ordinary non-async lexical arrow context; top-level await and new.target
retain real OXC syntax diagnostics. Authored nested async functions and ordinary
functions retain their own contexts. No generated arrow, parameters, body
container, Program or wrapper source is exposed as authored syntax.

SlotParams exposes authored formal-parameter items and a separate optional rest
parameter. Actual binding patterns, defaults, rest and TS annotations are
retained in OXC's AST. The generated arrow and parameter-list container stay
private. Empty bodies/lists contain no generated authored nodes.

Extraction requires a single arrow expression at the exact generated boundary,
the expected empty generated parameters or tail body, and authored top-level
node ranges. An input escaping the wrapper becomes `InvalidWrappedShape` and
cannot yield a successful body/list view. Exact AST projection retains the
existing UTF-8 and entity-boundary checks; diagnostics use the separate covering
projection. Comments and full OXC diagnostic metadata remain available when a
local hole hides the recovery AST.

The pinned OXC syntax parser accepts static module declarations in function
bodies and normally leaves their rejection to semantic validation. A complete
retained-AST visitor rejects these declarations, including nested bodies and
parameter defaults, with `InvalidModuleContext`. TS import-equals declarations
are also rejected. Dynamic import expressions remain admitted. This check does
not create synthetic OXC diagnostics, reparse source or construct a semantic
artifact. OXC syntax acceptance still does not prove all semantic early errors.
The only new normal dependency is the already pinned `oxc_ast_visit` library;
there is no legacy dependency, additional pipeline stage or serialization.

Ten native shape laws cover real return/directive/UTF-8 comment nodes, pattern
and default/rest spans, JS/TS annotations, empty shapes, trailing line comments,
once-decoded entity input and exact authored projection, retained real failure
metadata, lexical/module contexts, wrapper escape and wrapper-inclusive budget.
All 35 inherited source/syntax/admission and new shape laws pass in a scoped
`no_std` rustc harness using unchanged module copies and existing real pinned
OXC dependencies. Its cached L0 has the same required source API; this is scoped
module evidence, not a full current-workspace Cargo, Clippy or Actions claim.
Offline locked metadata, formatting, source-derived inventories and the
source-length ratchet also pass locally. Exact-head Actions and protected
queue delivery remain pending.

TODO:

- Construct ForHead and FilterChain from dialect-selected language pieces.
- Implement complete JS/TS recursion admission before accepting larger inputs.
- Admit JSX/TSX explicitly and resolve language once per file with mismatch
  diagnostics; attach artifacts to existing L0 node identities.
- Connect actual dialect, L2 and L4 consumers without reparsing; production
  route switches still require their product fix-history gates and native parity.
