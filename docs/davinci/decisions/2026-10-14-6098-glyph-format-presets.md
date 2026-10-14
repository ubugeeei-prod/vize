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

## CSS string quotes

Follow-up: `normalizeCssQuotes` (default `false`) re-quotes strings in CSS,
SCSS and Less `<style>` blocks with the `singleQuote` preference, as Prettier
does. A string keeps its quotes when its content holds more of the preferred
quote than of the other one. The pass skips comments, and in SCSS and Less it
treats every `//` as a line comment, so a doubtful `//` only leaves the rest of
its line unchanged. The pass runs before CSS layout, because the layout treats
`url('…')` and `url("…")` differently and running it afterwards was not
idempotent. The `prettier` preset sets it. Like the other new options, it is a
`FormatOptions` field that the WASM binding accepts; the Node binding and
the config-file key wait for the same loader follow-up as `preset`.

## Inline style attributes

The earlier sweep note that Vize adds a trailing `;` to `style` attributes was
wrong: Vize keeps the value as written, and Prettier lays it out as CSS.
`normalizeInlineStyles` (default `false`, set by the `prettier` preset) writes
a static `style` value as `name: value` declarations joined by `; `, without a
trailing `;`, and lowercases property names other than custom properties. It
only touches one-line declaration lists; a value with a newline, braces, a
comment, an entity or an escape stays as written. Breaking a long value over
several lines, as Prettier does, is not attempted.

## v-for parentheses and void elements

`normalizeVForParens` wraps several unparenthesized `v-for` aliases,
`item, i in items`, as `(item, i) in items`; a single alias, including a
destructured one, stays as written. `selfCloseVoidElements` closes the HTML
void elements (`img`, `br`, `input` and the rest, but never a capitalized
component such as `Input`) as `<img />`. Both default to `false`, keep the
compiled render code unchanged, and are set by the `prettier` preset.

## SCSS and Less layout

`reindentPreprocessorStyles` (default `false`, set by the `prettier` preset)
re-indents SCSS and Less blocks by their `{}` and `()` depth, adds one level
to a declaration continued over several lines, collapses blank lines and drops
them after `{` and before `}`. Only whitespace at line edges changes; lines
that start inside a block comment or a string stay as written, and an
unbalanced block is kept whole. Reformatting values, as Prettier's PostCSS
printer does, is not attempted: it needs an SCSS-aware value parser, and
Vize's CSS engine does not accept SCSS.
