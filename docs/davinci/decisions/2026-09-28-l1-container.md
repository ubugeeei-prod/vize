# L1 Vue container

Tracked in [#6837](https://github.com/ubugeeei-prod/vize/issues/6837).

- L1 owns a native SFC block scanner with script-aware and template-aware end
  search. The retained Croquis scanner remains its legacy product path until
  the compiler fix-history fixtures in #6880 are complete. This keeps legacy
  output and instruction counts unchanged while Davinci has no consumers.
- Attributes flow through an `AttrSink` in source order. `Vue::split` keeps
  every attribute as a source slice with its span, including duplicates.
- `Vue::split` reports `DuplicateBlock` for a second `<template>`, `<script>`
  or `<script setup>` and `MissingCloseTag` for a malformed block, then stops.
- A trial Croquis-to-L1 switch on PR #7039 failed the unchanged instruction
  ceilings in repeated exact-head Actions runs (20, then 15 overages); a
  `main` control passed. The consumer switch remains a separate, measured step
  after #6880. The ceilings only ratchet down.

TODO: line/column tracking still lives in the copied scanner; replace it with
the L0 line index when L1 becomes the production SFC path. Svelte, Analog and
TSRX formats are later `ContainerFormat`s.
