# Rare dialect input corpus (#6892)

The seven inputs under `tests/_fixtures/differential/dialect-inputs/` are
generated from a closed grammar table. Each row pins its source-rule witness,
Vue version and template-syntax selector, expected parser mode, stable input
ID and exact SHA-256. `--check` compares the complete generated file set,
input bytes and inventory with the grammar. Vue 0.10 and 0.11 intentionally
share template bytes but have distinct version selectors; template syntax
alone does not establish their different computed-property behavior.

This is a **product-neutral input inventory**, following #6891's shared
contract. Every row says `input-only`; it has no legacy or native observation,
equivalence result, fallback result or acceptance credit. A later product
manifest must reference its stable input ID and same bytes, add the product
target and actual adapter result, and keep unknown/unsupported rows in the
#6853 denominator and #6854 dialect bucket. The checked legacy source or test
name is a grammar witness, not a successful run of this generated input.

Remaining #6892 work: attach these inputs to executable compiler, linter,
formatter, typechecker and LSP cases as native paths arrive; add real pinned
Babel JSX and JS/TS/JSX/TSX case sources; classify each authored case and
measure #6854's zero-fallback stability period per dialect.
