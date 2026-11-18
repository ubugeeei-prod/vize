# A v-pre subtree keeps every element plain

Issue: [#13314](https://github.com/ubugeeei-prod/vize/issues/13314).

Vue's parser refines a tag type in `onCloseTag` only outside `v-pre`. The
element that carries `v-pre` and every descendant stay plain elements: a
component tag, `<component>`, `<slot>` and a structural `<template>` included.
Vize's SSR compiler already did this through its frozen-element custody, but the
ordinary parse kept the old classification, so DOM and Vapor resolved
`<div v-pre><Child/></div>` as a component.

## Decision

This partly supersedes the default-compatibility clause of the
[nested dynamic arguments record](./2026-10-10-ssr-nested-dynamic-arguments.md),
which restored the ordinary parser, Patina and every default lowering to the
shipped classification after the SSR literal `v-pre` fix. Only element
classification changes. Directive head names keep that record's split: the
compiler's full custody (`parse_with_frozen_elements`) keeps Vue's literal
names, and every other entry keeps the shipped names.

- Armature classifies a `v-pre` opener and its subtree as `Element` in every
  parse. HTML tree construction keeps reading the type the tag has outside
  `v-pre`, so a literal component neither leaves a table nor closes a `<p>`
  differently from its unfrozen control.
- DOM, Vapor and the NAPI template `compile` parse with
  `parse_with_frozen_element_spans`, which records the same frozen openings
  without directive head custody, and transform with them, so Core no longer
  promotes a literal element to a component. Vapor's lowering also keeps a
  literal `<component>` out of its dynamic-component path, and the legacy
  deletion guard follows the Vapor transform to `transform_with_frozen_elements`.
- The native L2 lowering decides component, slot outlet and element from the
  `v-pre` scope in every entry, not only under the compiler's frozen custody.
- Patina's markup facade and the L1 native lint tag facts classify frozen
  elements the same way, so lint rules that exempt components or slots now see
  a literal element.

The authentic-ca default packets of the literal slot boundary stay immutable.
`public-default-13314.corrections.json` beside them corrects only the code of
the six literal slot rows; both normal outlet rows keep their packets.

## Kept differences

A literal `<component is>` or `is="vue:…"` keeps its `is` attribute. Vue's DOM
compiler drops it there (and Vue Vapor drops it on `<component>`), while Vue's
SSR keeps it. A literal `<slot>` stays markup in Vapor, where
`@vue/compiler-vapor` 3.6.0-beta.10 also emits a `createSlot` call.
