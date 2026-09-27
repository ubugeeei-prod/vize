# L1 Vue container

Tracked in [#6837](https://github.com/ubugeeei-prod/vize/issues/6837).

- The SFC block scanner (script-aware and template-nesting-aware end search)
  moves from `vize_croquis` into `vize_l1::container::vue`. Croquis builds its
  legacy `SfcDescriptor` from `parse_block_fast`, so its output is unchanged.
- Attributes flow through an `AttrSink` in source order. Croquis keeps its
  map (a later duplicate wins); `container::vue::Vue::split` keeps every
  attribute as source slices with its span, so the container stays lossless.
- `Vue::split` reports `DuplicateBlock` for a second `<template>`, `<script>`
  or `<script setup>` and `MissingCloseTag` for a malformed block, then stops,
  matching where the legacy parser returns an error.

TODO: line/column tracking still lives inside the scanner for the legacy
descriptor; drop it once Croquis is replaced and positions come from the L0
line index. Svelte, Analog and TSRX formats are later `ContainerFormat`s.
