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

Retain the complete original #7895/#7902 reproductions and both follow-up reports
in the linter corpus. Runtime controls require exact warning ranges for unsafe
call results, unsafe callback callees and unknown assignments, while boolean
negation and typed callback results remain safe. PR shards disable the runtime;
full Check/protected execution with required Corsa is mandatory before delivery.
No dependency, performance budget, legacy-product replacement or native migration
claim is part of this change. Installed release validation remains with the
release owner.
