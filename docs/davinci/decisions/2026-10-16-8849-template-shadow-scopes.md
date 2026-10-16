# Template declaration scopes and authored ranges (#8849)

`vue/no-template-shadow` previously checked only `v-for` against the shared
parent-name stack and reported the whole directive. It missed nested modern and
legacy slot declarations. The expanded 138-project Matrix at
`d2506ca6b5e31252973f05981829c0367cacf26b` exposed 58 location pairs and 22 missing
template-ancestor warnings. All 609 reference warnings were reproduced from 226
complete pinned source files; the remaining 529 script/component-binding warnings
are a separate coverage decision and are outside this repair.

The rule now walks template lexical scopes with a local visible-name set and
push/pop log. It checks declarations in authored attribute order, including two
binding directives on the same owner, and restores visibility before visiting a
sibling. A shadowing declaration produces one warning even when several ancestors
share the name. This does not change other rules' shared scope metadata.

Modern slots consume their existing retained OXC parameter tree. A `v-for`'s whole
value is not an authoritative JS binding tree (`a in b in c` has a different JS
parse), and legacy slot attributes have no expression carrier. For these two
roles, the rule's declaration producer retains a direct original-slice parameter
goal in the current arena. The existing lowering's alias splitter supplies actual
borrowed alias positions, including omitted tuple positions; no wrapper source is
constructed. Failed or duplicate declaration groups contribute no partial names.
The existing parameter producer applies safety/depth guards and complete-input
checks. AST pattern walks collect binding identifiers, not property keys, default
references, or type names.

Both endpoints pass through the canonical Armature attribute entity map. The
complete decoded value must authenticate the authored source window before any
range is admitted. Escaped identifier width comes from the AST span, not decoded
name length. Named and numeric entities retain their complete authored extent.

The compatibility parser consumes semantic `v-pre`. To avoid changing its public
parse/retention behavior, the rule checks a candidate authored opening span with
the shared L1 lexer only when that span contains `v-pre`, then skips the subtree.
Text inside quoted values cannot impersonate a directive. Legacy `scope` applies
only to `<template>`; `slot-scope` applies to scoped-slot owners generally.

## Regression and evidence custody

`crates/vize_patina/tests/template_shadow_scopes.rs` checks complete public native
results for 44 positive/negative controls and two complete original n8n files.
The fixture records retain exact foreign answers, rule options and pinned runtime
versions, alongside independently declared native expectations. Script/component
controls explicitly retain their current no-warning result; no reference output
is rewritten to manufacture parity.

The controls cover same-owner order in both directions, tuple holes/trailing
commas/additional aliases, duplicate groups, object renaming, default/rest/array
patterns, siblings, own/inherited `v-pre`, quoted text, legacy tag restrictions,
Unicode and escaped identifiers, named and numeric entities, and invalid goals.
The original n8n cases retain the missing `message` slot in `MessagesList.vue` and
the correct nested `index` warning in `ChatMessageQueue.vue`, now at its identifier.
Original source revision/URL/SHA-256 and reference findings live in
`fixtures/template-shadow-scopes/original-sources.json`.

`matrix-packet-identities.json` retains the SHA-256 identities of all 41 complete
original comparison reports and 41 complete native linter envelopes. The original
82 packets and all 226 source receipts remain preserved with the read-only audit;
the older Matrix schema did not contain full raw foreign envelopes, so the audit
also retains fresh full pinned ESLint results for every original file. Neither
comparison normalization, diagnostic suppression, rule defaults nor Matrix budgets
change. The #8849 qualification umbrella remains open.
