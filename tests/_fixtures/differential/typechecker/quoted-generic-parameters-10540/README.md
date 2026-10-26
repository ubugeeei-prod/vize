# Quoted generic parameter boundaries (#10540)

All three projects are active batch fixtures and compare every returned production
diagnostic in order. `Comma.vue` retains the original quoted-comma constraint;
`Edges.vue` adds authored defaults, a template literal, a nested function type,
and a modifier comment containing a comma. The numeric parent must report one
TS2322 on its authored `value` key; repairing that value must restore an empty
vector without removing either child.

The negative expectation is independently authored from `reference.ts.txt`.
Read-only evaluation with the installed TypeScript 6.0.3 compiler returned the
single diagnostic `Type 'number' is not assignable to type '"a,b"'.` (TS2322).
Its Vue coordinate is the exact UTF16 start of the authored prop key, following
the existing named-prop fixture contract. This is source inspection and an
independent type-rule reference, not native qualification. Native batch Actions
must compare all three complete vectors before this pack is claimed qualified;
any discrepancy requires a causal mapping/type-rule investigation.

No historical fixture bytes, oracle providers, or diagnostic contracts change.
