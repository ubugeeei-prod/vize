# Typed template binding probes

Owners: [#7895](https://github.com/ubugeeei-prod/vize/issues/7895) and
[#7902](https://github.com/ubugeeei-prod/vize/issues/7902).

Use the authored initializer mapping for template expression and nested-callee
probes. A whole typed-check statement includes synthetic identifiers whose type
is the destination prop/listener type; those values may legitimately be `unknown`
for fallthrough attributes. They do not describe the author's expression.

Extend the existing inferred-expression binding pass to mapped component/native
prop initializers and single-expression handlers. Keep original checks and their
contextual typing intact. The copied probe stays in the original lexical scope,
and receives its own authored mapping. Reject malformed and multi-statement
handler bodies as full-expression probes while retaining existing call queries.
Do not borrow an unrelated expression binding from the preceding line.

Full Check on `de1a204` exposed the imported-component follow-up: `ChildPanel`
in dynamic `:is` still resolved to `any` because the Patina session did not load
its sibling SFC. Generate reachable relative/absolute Vue dependencies from their
actual sources with Canon's Options API projection, preserving authored default
types. Mirror them inside the private session and translate checker query offsets
through the import source map, including boolean batches. Leave missing or invalid
dependencies unresolved and retain unsafe authored component exports. Reuse
projections while their complete source text is unchanged, write only changed
mirrors, remove retired mirrors, and terminate cyclic dependency traversal.
Use the existing Patina projection helper for dependencies; skip dependency probe
bindings and discarded runner validation. The public editor-document facade fixes
`preserve_authored_component=false`, so it would replace an authored unsafe default
with a synthetic constructor. Dependency generation therefore selects the existing
Options API generator with authored-default preservation, including split scripts.
Escape rewritten module paths for the authored string delimiter in both relative
import projection and private dependency mirrors. Keep import-map adjustments in
bytes, then convert against the final text for UTF-16 checker queries. A projected
parent under a directory containing both quotes and an emoji must still parse,
resolve its actual child, and keep its query offset for either import delimiter.
The normal path without Vue imports avoids dependency projection and translation
allocations. Fresh Actions must prove the original corpus and exact unsafe controls
after this repair; the failing historical run grants no completion credit.

Retain the complete original #7895/#7902 reproductions and both follow-up reports
in the linter corpus. Runtime controls require exact warning ranges for unsafe
call results, unsafe callback callees and unknown assignments, while boolean
negation and typed callback results remain safe. PR shards disable the runtime;
full Check/protected execution with required Corsa is mandatory before delivery.
No dependency, performance budget, legacy-product replacement or native migration
claim is part of this change. Installed release validation remains with the
release owner.
