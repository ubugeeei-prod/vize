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

First exact-source run `37589426480` rejected a removed import still used by
the existing legacy-Vue completion helper. Restore that import without changing
its behavior or any fixture expectation; fresh successor Actions are required.

The identifier provider also runs without native support. Share the existing
pure member-position classifier across both builds (including its unchanged
Unicode/numeric tests), so non-native compilation and expression boundaries
retain the same contract; its OXC identifier dependency was already unconditional.

Source run `37590146726` passed the native-off/on stdio corpus and all four
tooling shards. Its actual non-native build emitted an unused import warning
for the legacy helper import, whose remaining caller is native-only. Match
that import to the caller feature gate; preserve all tests and require a fresh
successor run and warnings-as-errors feature checks before delivery.

The historical `9668561b43` Rust shards exposed exactly two whole-response
expression expectations in the existing native-event CLI tests: the original
reporter's interpolation coordinate and quoted event-handler expressions.
Keep the original four-candidate fixture byte-for-byte, then append the
separately frozen 41-candidate globals vector for those two calls. The full
response assertion remains exact; native tags, declared/native events, edits,
mirror observations, hover, and unsaved declaration controls retain their
original expectations. Require fresh successor Actions for both repaired
expressions and all four Rust shards.
