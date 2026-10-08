# Original local-alias transaction qualification

## Decision

Preserve the original `E.vue` and complete #8011 report, along with all six local
alias sessions delivered in #8181. Add separate frozen withDefaults and partial
destructure fixtures to qualify the remaining original-report forms using
whole public transactions. No production source or existing oracle changes.

The existing alias test skips `tone` for both nonplain forms and normalizes edit
entries before comparison. Its withDefaults form tests only the independent
bare `hidden` prop; its partial form destructures `tone`. Consequently it does
not establish the defaulted property's complete identity or the reported
`const { html = "" }` form with both `hidden` and `tone` bare.

The supplemental partial fixture adds optional `html` to the otherwise retained
local alias and destructures only that member. It keeps the reported original
bare `hidden` and `tone` expressions. The withDefaults fixture requires `tone`
to link its declaration, defaults key, and template use. Both fixtures have
independently authored complete repairs for each public property.

## Required observations

Across plain, withDefaults, and partial forms, every declaration and template
origin is tested with LF and CRLF. The defaults key is also an origin for `tone`.
This produces 26 fresh complete references/rename/application sessions and six
definition sessions covering the same 26 positions. Every session retains the
whole native TS2322 guard array and its clean repair before property requests.

Rename expectations precede provider queries. The unchanged authored-rename
helper compares whole references and WorkspaceEdit responses, applies actual
returned edits, checks complete file/disk bytes and version-2 diagnostics, then
installs an independent full golden and checks version-3 diagnostics. A missing,
foreign, duplicate, or malformed edit cannot disappear through normalization.
Definitions must resolve exactly to the property's authored alias-member span.

The original E.vue remains 192 bytes with SHA256
`7a32f47587a687f195b78597ffec552480594c928b6efc675715253a99413a36`.
The complete supplemental files and retained capture arrays are stored beside
the original corpus under `type-alias-navigation/8011/supplemental/`.

## Authority and delivery boundary

Use the unchanged strict ES2022/Bundler pinned-Vue helper and actual source-built
`vize lsp --stdio`. Captured TS2322 and repair establish each session's active
native checker, without claiming per-request backend-entry proof. Original
fixture bytes, runtime ownership controls, provider contracts, native assets,
400-file workload inputs, closed selectors, recipes, and numeric ceilings are
unchanged. The new test adds no production parser, query, or pipeline stage.

The slice depends on the existing #8218/#8233 Stack until its prefix actually
merges. Register it in the same native Stack and let the root coordinator admit
only an exact-head qualified contiguous prefix to the protected queue. Rebase
and retarget a remaining child only after actual prefix delivery. Fresh hosted
source/native qualification, protected execution, actual signed merge, and
installed release replay remain necessary. These finite contracts do not close
imported/complex alias, destructured-local, named-model, or stock ContentMapper
parity obligations.

No functional defect is asserted by the read-only coverage audit. Preserve any
authentic hosted failure and all authored expectations before correcting a
producer. The native guard, whole edits, diagnostics, and original fixture
identity must not be weakened to turn that failure green.

Paired issue decision: #8011. The canonical decision record receives the same
clause in this change through the root coordinator; no upstream write is made.
