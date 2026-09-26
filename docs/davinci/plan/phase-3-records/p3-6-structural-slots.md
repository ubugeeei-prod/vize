# Conditional and looped component slot content

The bounded native S3 contract covers `v-if` / `v-else-if` / `v-else`
and `v-for` directly on a component's named `<template #slot>` carriers.
Each branch or loop body must contain exactly one authored carrier. Ordinary
and dynamic components accept the contract; builtin wrappers retain their
separate runtime policies.

Conditional carriers emit one reactive selector with cached branch functions.
A dependency change at an unchanged branch/name preserves the mounted slot
body. Changing a name moves the slot to its corresponding child outlet;
missing carriers restore the child's fallback.

Looped carriers use the published Vue 3.6.0-rc.9 `createForSlots` helper.
The body receives reactive item/key/index refs, while name and optional identity
callbacks receive raw aliases. Array replacement, ordering changes and object
key/index changes update the refs without capturing stale loop values. Slot
props introduce their own lexical scope and can shadow loop aliases.

The IR keeps retained S2 expression trees for conditions, names, loop sources
and body expressions. Neither route reconstructs a JavaScript callback by
parsing synthetic source. The retained compatibility lowering creates the
same structural slot metadata for direct code and mapping comparisons.

## Validation

- `s3::tests::structural_slots` compares both prefix modes, checks payload
  ownership, and refuses implicit-content mixtures, destructured loop aliases,
  nested carriers, combined condition/loop carriers and builtin slot controls.
- The parser agreement corpus includes the supported sources and their
  prefixes/punctuation deletions.
- `tests_source_map::structural_slots` records exact code and decoded mappings.
- `davinci_expr_reparse_floor` keeps native walk/reparse counts at zero and
  leaves the seven allocation ceilings unchanged.
- `davinci_structural_slots_parity` runs separate native, retained and official
  mounted traces. It checks conditional name changes, array/object reorder,
  item replacement, insertion/deletion, fallback restoration, scoped props,
  events, node identity and final unmount. Its retained compiles run in their
  own binary so they cannot race native-only process-global walk probes.
- The mounted binary is required by the Lean workflow and its tooling contract.

These proofs cover this bounded slot surface. Nested conditional/loop carriers,
complex parameter patterns and builtin wrapper policies remain independent
contracts; the broader P3-6 exit is still open.
