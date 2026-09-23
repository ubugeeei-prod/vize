# Style specification

**Status:** proposal for maintainer sign-off ([charter #41](../README.md#decisions)). No earlier phase pre-committed a direction. This file is the blank-slate discussion, written as output rather than adjectives. [P4-12b](./phase-4-tasks-last.md#p4-12b--glyph-on-s1) implements it. Pug layout is [P4-12c](./phase-4-tasks-last.md#p4-12c--pug-as-an-s1-dialect) and is out of scope here.

**Review point (open):** the maintainer signs the direction. Until that happens the fixtures are the proposal on the table, not a compatibility promise with today's Glyph.

## Oracle

Each decision is a heading `### Snn — Title` (`S` and two digits). Its fixture pair is `crates/vize_glyph/tests/style_spec/Snn/{input,output}.vue`. `output.vue` is the whole file under **default options**, already obeying every rule, not a partial diff of the rule it illustrates. `input.vue` is legal Vue and differs only by the decision that rule names, as far as that can be done in one document.

`node --test tests/tooling/davinci-style-spec.test.ts` is the bijection: every heading has both files, every pair has one heading, and an injected orphan pair fails. It does not run the formatter. P4-12b's `cargo test -p vize_glyph --test style_spec` will require `format(input) == output` and `format(output) == output`, and TS-5 (idempotence, parse-preservation, lint-agreement, pug) stays in force. Lint-agreement allows a finding's count to shrink and forbids it to grow.

Columns, unless a rule says otherwise: print width **100**, indent **two spaces**, end of line **LF**. Script and directive expressions use the shared JavaScript defaults: semicolons, double quotes, trailing commas wherever a list actually breaks, spaces inside `{}`, parentheses on a single arrow parameter. Plain CSS uses the same indent. `vueIndentScriptAndStyle` is false. Those options may keep existing names; TS-41 pins the defaults only.

## Direction

**Side-effect-stable canonical print.** Three commitments, in order:

1. **Do not rearrange runtime text or effectful attributes for looks.** Whitespace Vue's default compiler keeps as a text node stays. A directive, a binding (`:`, `@`, `#`, a `.` prop), and a literal value that contains `{{` are order barriers.
2. **What the linter requires, print. What it merely permits, do not churn.** Static attributes move only inside a barrier-free segment, by `vue/attribute-order`'s categories, authored order within a category (the lint's alphabetical switch defaults off). Top-level blocks stay in authored order (`vue/sfc-element-order` accepts script-then-template and template-then-script). The formatter is not that lint's autofix.
3. **One printing** for quotes, shorthands, slots, self-closing, mustaches, and the envelope.

Departures from Glyph as it prints today, on purpose: blocks are not reordered (`sort_blocks` default becomes false); attributes are not alphabetized and binds are not pulled in front of literal attributes in the same category; `v-slot` follows `vue/v-slot-style` instead of a blanket `#` rewrite; void elements and empty components self-close.

## Rules

### S01 — Line endings

The file uses LF only. It ends with exactly one LF. No line has trailing spaces. A CR or a missing final newline is layout, not text.

### S02 — Block spacing

Consecutive top-level blocks are separated by exactly one blank line. There is no blank line before the first block (a prologue uses S04) and none after the last.

### S03 — Block order

Top-level blocks stay in authored order, including template-before-script and a `<style>` that the author placed before `<script>`. Two `<style>` blocks keep their order (cascade and `@import` are semantic). Custom blocks stay where they were written. Reordering to script-then-template is rejected: it rewrites the template-first half of the corpus without a lint that requires it.

### S04 — Prologue

Bytes before the first top-level tag, trimmed, are kept, then one blank line, then the blocks. A license comment is not a block and is not moved below `<template>`.

### S05 — Block body indent

`<script>` and `<style>` bodies start at column zero. The tags are not an indent level (`vueIndentScriptAndStyle` false).

### S06 — Script printing

A `<script>` / `<script setup>` body is the JavaScript printer at the shared defaults, including `lang="ts"` / `tsx`. The fixture is the pinned printing of that program. A language the JS printer cannot parse is left trimmed, not dropped.

### S07 — Block attributes

On `<script>`, `setup` then `lang` then every other attribute A–Z. On `<style>`, `scoped` then `lang` then the others A–Z. On `<template>`, `lang` then the others A–Z. On a custom block, all attributes A–Z. Values use S22. `setup`, `scoped`, and `module` stay valueless booleans when they were valueless.

### S08 — Style languages

`lang` absent or `css` is printed as CSS (the fixture is that printer's result). Any other `lang` is trimmed and otherwise copied, so SCSS nesting is not fed through the CSS printer.

### S09 — Opaque templates

A `<template lang>` other than `html` (comparison is ASCII case-insensitive; absent `lang` is HTML) is not parsed as HTML. Only the shared leading indent of its non-blank lines is rebased so the body sits one level under the tag. Relative indents, blank lines, and `{{ }}` bytes stay. Pug layout is not this rule.

### S10 — Template indent

HTML template children indent two spaces per element from the `<template>` tag. The tag itself is at column zero.

### S11 — Self-closing

HTML void elements always self-close. An empty component (a tag that is not HTML, SVG, or MathML) self-closes. An empty SVG or MathML element self-closes. A normal HTML element always uses a separate end tag, including when empty (`<div></div>`, never `<div />`): in HTML the slash is not an end. One-line self-closing tags have a space before `/>` (`<br />`). A self-closing tag broken by S12 puts `/>` on its own line in the tag's column, with no extra space. A component that has children keeps its end tag.

### S12 — Opening-tag wrap

`printWidth` 100 is measured on the template body's own columns, before the SFC adds the two spaces from S10. Two or more attributes break when the single-line start tag would pass column 100. One attribute never breaks the tag, even past 100 (breaking it cannot shorten that line). A value that already contains a newline always breaks the tag. `bracketSameLine` is false: `>` goes on the next line, in the tag's column, and attributes indent one level. A child that was source-adjacent to `>` stays on that `>` line (S17); the fixture's child is separated by a newline, so `>` is alone.

### S13 — Static attribute order

Inside a run of static literal attributes with unique names (ASCII case-insensitive), stable-sort by `vue/attribute-order`: `is`; `id`; `ref`, `key`, `slot`, `slot-scope`; then every other literal attribute. Same category keeps authored order. Do not alphabetize (`class` before `title` stays if it was written that way). A repeated name freezes the whole run. The category list's directive groups are recorded so a static `is` / `id` / `ref` agrees with the lint; directives themselves do not move (S21).

### S14 — Directive shorthands

`v-bind:arg` becomes `:arg`, `v-on:arg` becomes `@arg`, including a dynamic argument (`v-bind:[name]` → `:[name]`). Argument-less `v-bind="row"` and `v-on="row"` stay. Modifiers stay in authored order and stay attached (`.prevent.stop` is not sorted). Values are not rewritten into valueless attributes: `:disabled="true"` stays, because on a non-boolean prop the valueless form is `""`, not `true`. Slot spelling is S15, not this rewrite.

### S15 — Slot spelling

Match `vue/v-slot-style`. The default slot on a component is `v-slot` with no argument (`<Layout v-slot="slotProps">`, not `#default`). The default slot on `<template>` is `#default`. A named slot, on a component or a `<template>`, is `#name`. A blanket `#` rewrite introduces that lint; this rule exists so the shorthand cannot.

### S16 — Mustache spacing

An interpolation is `{{ ` + the JS printing of its expression + ` }}`, one space inside each delimiter. Text outside the delimiters is not padded to make the mustache pretty (S17).

### S17 — Text adjacency

A text node is preserved. Horizontal whitespace between inline siblings condenses to one space. A whitespace-only gap that contains a newline is Vue's condensed gap: it becomes the layout break, not a space, and must not be invented where the source had no whitespace (adjacent tags stay adjacent, including a child glued to `>`). Leading or trailing spaces inside an element that are not a newline gap stay (`<p> Hello </p>`).

### S18 — Significant whitespace

The bytes inside `<pre>`, `<textarea>`, `<listing>`, and any element with `v-pre` are copied. The SFC indent is not applied to those lines (a second pass would walk them right forever). Attributes on the element's own start tag still follow S13 and S14. `{{ }}` inside `v-pre` is literal text, not S16.

### S19 — Comments

An HTML comment is kept byte for byte, in place. No space is inserted or removed around it. Mustache spacing still applies on either side.

### S20 — Directive expressions

A directive or binding value is the JS printing of that expression, then S22 chooses the quotes. Modifier order is authored (S14). The fixture is the one-line case; if the JS printing contains a newline, the value's lines are that printing unchanged and the tag breaks under S12.

### S21 — Order barriers

A directive, a binding, or a literal value containing `{{` does not move, and a static attribute does not cross one. Statics in the open segment between barriers still follow S13 (`:value class id :next` can become `:value id class :next`). Side-effect order outranks `vue/attribute-order`: a `v-if` written after `class` stays after `class`, and the leftover warning is accepted. Introducing a warning is not.

### S22 — Attribute quotes

Double quotes. A value that contains `"` and no `'` uses single quotes. A value that contains both keeps double quotes and escapes `"` as `&quot;`. Directive expressions are quoted after the JS printer runs, so a printed `"` flips the attribute to single quotes.

### S23 — Suppression lines

A physical line covered by `eslint-disable-next-line`, `vize-disable-next-line`, `@vize:expected`, `@vize:level(…)`, `eslint-disable-line`, or `vize-disable-line` is not split, even past print width. Splitting moves the code off the line the pragma covers and the finding comes back. Shorthand and quote rules still apply on that line. The covered line's own spaces follow S17; the pragma comment follows S19.

### S24 — Custom blocks

A custom block that is not `art` is trimmed and otherwise copied (an `<i18n>` JSON document is not pretty-printed). An `<art>` block's chunks are HTML template printing, indented one level under the tag, with a blank line between chunks preserved. Block order is still S03.
