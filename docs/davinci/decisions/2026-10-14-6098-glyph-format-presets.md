# Formatter presets (#6098)

Issue: [#6098](https://github.com/ubugeeei-prod/vize/issues/6098).

#6098 asks for named formatter profiles, starting with a Prettier-like
migration profile. A sweep of 3,515 SFCs from six real projects against Oxfmt
(Prettier-compatible) found that most differing files came from four Vize
defaults: sorted template attributes, sorted SFC block attributes
(`<script lang="ts" setup>` became `<script setup lang="ts">` even with
`sortAttributes: false`), canonical block order and directive shorthand
normalization. The rest are intended differences listed in the formatter
guide under "Coming from Prettier".

Decision: a preset is a named assignment of existing options, not a new
formatting mode. `vize_glyph::FormatPreset` has two values. `vize` assigns the
Vize defaults. `prettier` sets `printWidth: 80`, `sortAttributes: false`,
`sortBlocks: false` and `normalizeDirectiveShorthands: false`. A preset owns
only those four options and never adds a code path of its own, so every preset
output is an output that some explicit option combination already produces and
is covered by the same idempotence tests and corpus sweeps.

The public name is "preset", because `vize fmt --profile` already prints a
timing profile. The CLI takes `--preset <vize|prettier>`. Options resolve in
this order: config file, then the preset, then individual CLI flags.
`sortAttributes: false` now also keeps the authored attribute order on
`<script>`, `<template>`, `<style>` and custom block tags, so the Prettier
preset matches Prettier on those tags.

Not decided here: a `preset` key in `vize.config.*` and `fmt.vize`. Config
values arrive at the CLI already merged with defaults, so honoring "explicit
config wins over preset" needs the loader to track which keys were set. That
change touches the Pkl schema, the JSON schema and the generated TypeScript
types and is left to a follow-up on #6098. Presets for compact application
code and documentation examples are also left open; they should be added only
with corpus evidence that a fixed option set serves them.
