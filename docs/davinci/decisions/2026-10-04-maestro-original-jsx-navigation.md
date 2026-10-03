# Original JSX/TSX Maestro navigation

Tracked in [#6871](https://github.com/ubugeeei-prod/vize/issues/6871) and
[#6883](https://github.com/ubugeeei-prod/vize/issues/6883).

## Decision

The opt-in `vize/nativeDefinition` and `vize/nativeReferences` workers now
select JSX or TSX from the actual `javascriptreact` or `typescriptreact`
Document language. The private profile retains the complete original
`ProgramOptions`: JS/TS, JSX permission and the explicit Module goal. A URI,
source spelling or parse retry cannot supply that permission. Plain JS/TS and
the existing Vue source configuration retain their original profiles.

The same worker constructs one original L1 Program per physical immutable
snapshot and gives its authentic admitted observation to the existing sole
L2 File producer. That producer already records genuine JSX opening component
and member-root references, expression containers and spreads. It does not
invent occurrences for intrinsic names, static properties or closing tags.
Requests query the same retained File bindings and recorded references;
neither a copied index, second parser nor legacy resolver is introduced.
Original TS annotations remain part of the retained Program. Navigation needs
no runtime erasure, emission or Vue unwrapping authority.

The current snapshot/profile cache, bounded mailbox and native owner lifetime
are unchanged. URI/global revision/client version/language and physical Arc
identity guard publication. Host mutations retire superseded workers; request
cancellation and ready-response checks remain authoritative. Close, reopen,
project drop and unwind release original owners normally without lifetime
casts, leaks, joins under locks or a new pipeline stage.

## Verification and remaining work

Nine new project laws and four complete production-service JSON-RPC Value
laws are source-authored, with zero executions before hosted qualification.
They cover both actual react language IDs, explicit TSX Module admission,
component/member/container/spread targets, static-name and closing-name
negatives, original types and UTF-16/shadowing, sticky syntax/File refusals,
one original parse across repeated queries, foreign/stale/language replacement,
queued and ready cancellation, close/reopen and owner cleanup.

The existing native-navigation action executes these laws on affected Maestro
PR source and unconditionally in the protected full Rust/differential recipe.
It also checks the minimal feature and denies all-targets feature warnings.
Exact-head Actions, actual all-100 instruction ceilings and terminal protected
merge acceptance are still required. No local Rust build or extra manual full
campaign supplies credit.

This bounded local navigation extension does not replace standard LSP routes.
Workspace/external navigation, hover/type definitions, L3/L4 retention,
arbitrary JS/TS/JSX/TSX semantic coverage, default migration and the complete
LSP fix-history/state contract remain unfinished. The roadmap issues stay open.
