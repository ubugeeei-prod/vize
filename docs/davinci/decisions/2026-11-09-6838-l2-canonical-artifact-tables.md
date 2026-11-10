# L2 canonical artifact tables

Tracked in [#6838](https://github.com/ubugeeei-prod/vize/issues/6838), with the
DOM half of [#6840](https://github.com/ubugeeei-prod/vize/issues/6840) as its
consumer.

## Template wrappers become fragment-carrier facts

The transitional lowering kept two Vue-named side tables beside the op tree:
`wrappers` (`<template v-if>` keys, which branches unwrapped, a chain-wide
`v-once`) and `for_wrappers` (`<template v-for>` key, attributes, class and a
refused runtime directive). The DOM and SSR emitters and the `v-slot` grouping
read them.

What they record is not Vue policy. A conditional branch or loop body that is
rendered through an unwrapped carrier instead of a single root is the same
thing to every backend, whether the input authored `<template v-if>` or any
other fragment carrier. The schema therefore moves to
`vize_l2::analysis::fragment_facts` under neutral names:

| Former lowering type | L2 fact           |
| -------------------- | ----------------- |
| `WrapperKeys`        | `BranchFragments` |
| `WrapperKey`         | `FragmentKey`     |
| `ForWrapper`         | `LoopFragment`    |
| `WrapperAttr`        | `FragmentAttr`    |
| `WrapperClass`       | `FragmentClass`   |

`BranchFragments::from_template` becomes `fragment`. Deciding which surface
unwraps, capturing its key and attributes, and recording provenance and
dropped attributes stay in the Vue lowering (`lower/structural/wrapper.rs`).
This is how the issue's "dialect-derived tables are absorbed by dialect
legalization" is met for wrappers: legalization produces neutral facts, and
no Vue-named table reaches the canonical artifact.

The rename is a schema move only: layouts, sizes, keys and every emitted byte
are unchanged.

## Target shape

The remaining #6838 slices follow from the same rule:

- The canonical artifact is the `vize_l2::artifact` owner: ops plus semantic
  side tables (scopes, provenance, compound text, `ui.for` and `ui.if` facts,
  fragment carriers, op-family features). Conversion residue (arena, minted
  count, diagnostics, Vue capabilities) stays on `vize_l1_to_l2::Lowered`.
- Backend-neutral passes and their fact products live in `vize_l2`; Vue 2
  sugar legalization stays in `vize_l1_to_l2`.
- The DOM emitter reads only `vize_l2` types, which lets it move into
  `vize_l4::targets::dom` without `vize_l1_to_l2` internals.

## The canonical artifact owner

`vize_l2::artifact::ArtifactParts` is the one canonical artifact. It already
carried the source, ops, provenance and scopes for native producers; it now
also carries compound text, `ui.for` and `ui.if` facts, both fragment-carrier
tables and the op-family `LoweringFeatures` (moved from the lowering
unchanged). It is generic over the provenance sink, so the transitional
lowering keeps its record-free sink, while `Artifact::try_new` still seals only
the record-keeping default and now also rejects a dangling fact key.

`vize_l1_to_l2::Lowered` holds that artifact as `artifact` beside the
conversion residue that never leaves the crate: the arena, the minted op
count, diagnostics, lint-only consumed expressions and Vue capabilities.
Consumers read `lowered.artifact.*`; there is no second copy and no extra
walk, so construction cost is unchanged.

## Pass ownership after the DOM move

Backend-neutral pass bodies and every fact schema they publish live in
`vize_l2`: static-subtree and constant-expression analysis, template
complexity, compound text, `ui.for` / `ui.if` facts, slot groups, model
faults, fragment carriers, and the `L2Facts` product
(`vize_l2::analysis::pass_facts`). Vue 2 filter assets are a neutral
`FilterAssets` row there, not a legacy table.

`vize_l1_to_l2::pass` keeps only Vue legalization over its own lowering
result: Vue 2 sugar (`legacy-sugar`), the `v-slot` grouping with Vue's four
diagnostics, `v-model` scope validation, and the artifact-scoped plan that
orders them before the L2 analyses. These are the "legalization patterns" the
issue leaves in the conversion crate; the DOM emitter no longer lives there.
