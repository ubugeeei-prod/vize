# Template expression globals (#8015)

Issue: https://github.com/ubugeeei-prod/vize/issues/8015

Vue expression identifier completion previously returned only authored script,
props and template-scope bindings. Append the 14 named public instance properties
and methods plus the 27 JavaScript names permitted by Vue 3.5.43. Existing authored
items keep their whole type, documentation, sort, insertion and resolve payload
when they shadow a built-in. Built-ins sort after authored bindings.

Use the existing template analysis once. No new parse, checker request, cache,
pipeline stage or dependency is introduced. `$event` remains event-handler-local.
The public instance's internal `$` object and arbitrary browser/Node names are
not added. Petite-vue retains its current scope surface. The existing checker
continues to own member access; this change supplies identifier candidates.

Retain the complete original Issue body and all four original SFCs in the legacy
LSP differential corpus. A frozen whole completion bank covers the original
`$e`, `St` and `Ma` positions, no-setup and unquoted directive values. Rust and
production stdio controls exercise authored binding/prop/loop shadowing, literal
attributes/comments, `$event` scope, resolve, unsaved Unicode/CRLF and restoration.
The stdio corpus runs with type checking disabled and enabled. Its physical Vue
and TypeScript versions are the repository's test dependencies; the report's
Vue 3.5/TypeScript 5.9 environment is retained as original evidence.

Validation status: source preparation; exact-head Actions, protected full suites,
unchanged instruction ceilings, actual merge and installed release replay remain
required. Issue #8015 stays open for the remaining attribute and generic hover
work. PR #8131's partial common-attribute change is independent and untouched.

References: [Vue allowed globals](https://github.com/vuejs/core/blob/v3.5.43/packages/shared/src/globalsAllowList.ts)
and [public instance surface](https://github.com/vuejs/core/blob/v3.5.43/packages/runtime-core/src/componentPublicInstance.ts).
