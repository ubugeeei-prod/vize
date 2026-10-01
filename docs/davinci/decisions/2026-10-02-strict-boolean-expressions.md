# Opt-in strict boolean expressions

Issue: [#7377](https://github.com/ubugeeei-prod/vize/issues/7377).

Add `type/strict-boolean-expressions` to the opt-in catalog without changing
any preset. Native type-aware lint uses the existing Corsa project and Canon
projection for both script blocks and template `v-if`, `v-else-if`, and `v-show`.
Conditions include if/while/do/for/ternary tests, negation, and logical operands;
the right operand of a value-producing logical expression is not a condition.

Use position types with control-flow narrowing, checker assignability,
non-nullable types, and intrinsic/literal type handles. Do not infer aliases,
object unions, or nullability by spelling or by an identifier's declared type.
Opaque handles stay within their snapshot. Reuse Corsa's existing strict boolean
policy with checker-derived condition facts and preserve authored byte ranges.

Typed options are `allowString`, `allowNumber`, `allowNullableObject`,
`allowNullableBoolean`, `allowNullableString`, `allowNullableNumber`, `allowAny`,
and `allowNullableEnum`. The first three default to true and the rest to false,
matching the upstream rule. Configuring options alone does not enable the rule;
explicit `off` and severity/entry overlays keep their usual precedence.
The current native checker projection already forces strict checking. This
slice does not claim assertion-function or array-predicate ESLint parity.

CLI, LSP, Pkl, JSON schema, and generated TypeScript must share these options.
All three diagnostic catalogs and complete explain-page snapshots include the
new rule; the catalog total increases from 250 to 251 without changing presets.
Retain the stable public exhaustive config structs using additive wrappers.
Move existing probe execution and option declarations in a move-only commit
before implementation to keep the source-size ceiling unchanged.

Authored fixtures cover the optional DOM element repro, nullable primitives,
boolean/never/any/unknown, aliases and object unions, control-flow narrowing,
logical value contexts, template conditions, exact spans and option overrides.
Run source-built native contracts in Actions and retain all existing numeric
instruction and divergence limits.
