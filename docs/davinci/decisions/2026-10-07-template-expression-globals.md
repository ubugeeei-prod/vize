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

Source run `37592198349` then compiled the actual structural `--lib` test
configuration and exposed the import's missing test arm. The legacy helper
caller is guarded by `any(test, feature = "native")`; match that exact gate on
the import. This keeps structural production builds warning-free and retains
existing structural unit tests. The initial native-only gate was incomplete;
require both actual structural tests and production checks on the successor.

The backlog refresh genuinely fetches main
`c86c7a21f9b858122c0aa42f9771555e33a87c32` after local, remote and PR ownership
checks of the original `64b79c2ab0` head. Replay only this PR's six commits;
retain all incoming decision bytes and the 350-line canonical record. Preserve
the now-merged common-attribute modules, checkout rules and whole native-event
vectors alongside the globals extension. Standalone production/tests and the
original corpus stay byte-identical. The previous head's successful source
and native receipts are historical; require fresh exact-head source/native
Actions on the genuine union before root-owned cohort admission and release.

The next refresh genuinely fetches actual main
`3ed1cc90908c88016310b90a855500c94a156f1f`, after rechecking local, remote and
PR ownership at `9c5db6fcb79ea526a5dada6c94b4d786fa3065ce`. Replay the seven
existing commits through observed actual `8c772761`, then integrate only the
fetched actual `3ed1cc90` ancestry. Prospective cohort analysis objects never
supply source ancestry. Preserve all 15 owned source/test/inventory/corpus
blobs, all 33 original native-event/common-attribute controls, the frozen whole
41-global bank and the exact native-or-test helper gate. Retain incoming
checkout rules and the actual main native workflow byte-for-byte at 350 lines.

Use the root-approved common canonical file unchanged, SHA-256
`f4bd4c030fe794c89c7872cdfdcfaa3823d1dc93bdd44295b83fce11637b570d`, at 350
lines; it retains the incoming, prospective and historical owned clauses.
Previous `9c5db6f` source, native, production stdio and 16,546 selected Rust
passes remain historical. Fresh exact-head automatic source/native Actions
must qualify this union before root-owned queue admission and release.
No installed replay, whole-Issue completion or measured speed claim follows
from the local refresh.

## Full editor oracle follow-up

The Element Plus badge cursor is `{{ val|ue.toUpperCase() }}` and the packaged
Vim cursor is `<Child  :count="|total" />`; both request expression identifiers.
The 41 Vue instance and allowed JavaScript globals added for #8015 are valid at
these positions. Preserve the original ordered Element Plus `value`/`Message`
answers and the complete Vim `Child`/`total` objects, then explicitly append all
41 frozen global answers. Retain every original authored source, request position,
hover, diagnostic, revision, formatting, code action and semantic-token control.
No provider route or assertion is weakened; full Check must replay both scenarios.

The [current `a5c09ab` full run](https://github.com/ubugeeei-prod/vize/actions/runs/37647333345/job/112881429585)
passes the complete Vim and Neovim scenarios, then exposes the same inherited
expression-answer omission in the shared Rust Zed/Helix smoke oracle. Preserve
its two complete original `Child`/`total` objects and every original source,
position and other response, then append the complete already-frozen global bank.
The Rust runner and its two compatibility mirrors use that immutable corpus;
no actual response supplies expected fields. The Zed failure and consequent
Helix skip remain recorded until fresh exact-head full qualification executes both.
