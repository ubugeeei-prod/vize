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
- The exact-head instruction-count gate exposed a cross-crate cost after the scanner
  moved from Croquis to L1: 20 pinned Croquis and atelier cases exceeded their
  unchanged ceilings on head `576baa9b3`. A plain `#[inline]` hint gave
  byte-identical counts on head `a05640f2a`; a control run on `main` passed all
  ceilings. Forcing cross-crate inlining still exceeded 20 ceilings on
  `f5c520422`, so the parser now reuses the already-found interpolation close
  in the slow path and makes the legacy attribute map adapter inline. Measure
  this on the exact PR head before re-entering the queue; ceilings stay fixed.

TODO: line/column tracking still lives inside the scanner for the legacy
descriptor; drop it once Croquis is replaced and positions come from the L0
line index. Svelte, Analog and TSRX formats are later `ContainerFormat`s.
