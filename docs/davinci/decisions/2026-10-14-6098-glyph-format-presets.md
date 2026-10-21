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
formatting mode. `vize_glyph::FormatPreset` has four values. `vize` assigns the
Vize defaults. `prettier` assigns `printWidth: 80`, authored attribute and block
order, long closing brackets and the Prettier-compatible normalization options.
Each preset only assigns existing options and never adds a code path of its own,
so every preset output is an output that some explicit option combination already
produces and is covered by the same idempotence tests and corpus sweeps.

The public name is "preset", because `vize fmt --profile` already prints a
timing profile. The CLI takes `--preset <vize|prettier|compact|documentation>`. Options resolve in
this order: config file, then the preset, then individual CLI flags.
`sortAttributes: false` now also keeps the authored attribute order on
`<script>`, `<template>`, `<style>` and custom block tags, so the Prettier
preset matches Prettier on those tags.

The `preset` config key was left to a follow-up; see below. Compact application
and documentation-example profiles are recorded below.

## CSS string quotes

Follow-up: `normalizeCssQuotes` (default `false`) re-quotes strings in CSS,
SCSS and Less `<style>` blocks with the `singleQuote` preference, as Prettier
does. A string keeps its quotes when its content holds more of the preferred
quote than of the other one. The pass skips comments, and in SCSS and Less it
treats every `//` as a line comment, so a doubtful `//` only leaves the rest of
its line unchanged. The pass runs before CSS layout, because the layout treats
`url('…')` and `url("…")` differently and running it afterwards was not
idempotent. The `prettier` preset sets it. Like the other new options, it is a
`FormatOptions` field that the WASM binding and the config file accept; the
Node binding accepts it together with the other four migration normalizers
below. Explicit Node values are applied after a preset, including `false`.

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

## The `preset` config key

`vue.formatter.preset` (or `fmt.vue.preset` under Vite+) selects a preset in the config file. The table of assignments
now lives in `vize_l0::config::FormatterPreset::options`, keyed as in config
files, and `vize_glyph::FormatPreset` re-exports it, so the CLI flag, the
Node API and the config key share one table. The loader no longer merges a
formatter section with defaults before the preset is known: it overlays the
authored keys on the preset's options, and only then fills the stable
`FormatterConfig`. `ConfigDocument::formatter_profile` keeps the merged
object, which also carries the options outside `FormatterConfig`
(`normalizeCssQuotes` and the rest), and `FormatOptions::from_profile` turns
it into native options for the CLI and the language server. This replaces two
hand-written conversions that set those options to `false`.

Order: the config preset, the other config keys, `--preset`, the other CLI
flags. Pkl's `FormatterConfig` options are now nullable, because a Pkl
default would otherwise always count as an authored key; null values are
treated as unset. A legacy `fmt` section is used only when `formatter` has no
keys at all.

## Compact application layout

The `compact` preset is a named assignment of existing options, not a new
printer mode. It sets `printWidth: 120`, `bracketSameLine: true`,
`sortAttributes: false` and `sortBlocks: false`. Directive shorthands stay
enabled, while the Prettier-only CSS, inline-style, `v-for`, void-element and
preprocessor normalization switches are reset to `false`.

The fixed fixture under `crates/vize_glyph/tests/fixtures/profiles` proves the
denser tag layout, authored attribute and block order, and byte-identical
second pass. Config, CLI and native Node tests cover the public typed paths and
override order. The JSON Schema, generated TypeScript declaration and Pkl
declaration expose the same four-value enum.

## Documentation examples

The `documentation` preset keeps examples at `printWidth: 80`, places one
attribute on each line, preserves authored attribute and block order, and
keeps directive shorthand normalization. It leaves the five Prettier-only
normalization switches off. The fixed fixture under
`crates/vize_glyph/tests/fixtures/profiles` proves the intended complete output
and its second-pass fixed point. The public config, CLI and Node paths use the
same shared preset assignment, and every preset now runs over the 300+ file
option-matrix corpus idempotence gate.

## Node normalization options

The native Node `formatSfc` options expose `normalizeCssQuotes`,
`normalizeInlineStyles`, `normalizeVForParens`, `selfCloseVoidElements` and
`reindentPreprocessorStyles` directly. They use the same booleans as config,
the CLI's resolved profile and WASM. An explicit Node value is applied after
the selected preset, so every Prettier migration normalizer can be enabled
without the preset or disabled with `false` while retaining the rest of it.

## Script quote option parity

The existing `jsxSingleQuote` boolean and `quoteProps` enum now have direct CLI
and native Node projections in addition to Rust, config schema, generated
TypeScript config, editor and WASM support. CLI flags are
`--jsx-single-quote[=false]` and
`--quote-props <as-needed|consistent|preserve>`; Node `formatSfc` uses the
camel-case option names and rejects unknown property-quote modes at the native
boundary. The pre-existing independent full-output fixtures cover every
property mode and every JavaScript/JSX quote combination across Vue versions;
public CLI and Node regressions add second/third-pass fixed-point evidence.

## Trailing comma option parity

The existing `trailingComma` enum now has a direct CLI override and native
Node projection in addition to Rust, config schema, generated TypeScript,
Pkl, editor and WASM support. The CLI flag is
`--trailing-comma <all|es5|none>` and Node `formatSfc` uses the same camel-case
name and three values. Explicit values are applied after a preset, and unknown
Node strings fail at the native boundary. Public CLI and Node whole-output
regressions prove a second/third-pass fixed point; the existing option matrix
continues to exercise the `es5` and `none` axes over the 300+ Vue corpus.
