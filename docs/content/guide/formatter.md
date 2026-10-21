---
title: Formatting
description: Format Vue components with Vize, alongside Oxfmt for the rest of a Vite+ project.
---

# Formatting

Vize formats the `<script>`, `<template>` and `<style>` blocks of a Vue component in one pass. In a Vite+ project it handles `.vue` files and leaves every other file to Oxfmt, so one command formats the whole project.

## Format a project

With the [Vite+ integration](./vite-plus.md):

```bash
vp run fmt:check
vp run fmt
```

`vp run fmt:check` reports the files that would change and fails if there are any; use it in CI. `vp run fmt` writes the result. Pass paths after `--` to format part of the project.

Without Vite+, call the [CLI](./cli.md#format) directly. It formats Vue, JavaScript, TypeScript, JSX, TSX, JSON and JSONC files. YAML and Markdown are not formatted.

```bash
vize fmt --check src
vize fmt --write src
```

`--check` lists each file and exits with status 1 when a file needs formatting:

```text
Found 1 file(s)
Would reformat: src/Card.vue

Checked 1 file(s)
  1 file(s) would be reformatted
```

## What changes

Given this component:

```vue
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ title: string; count?: number }>();
const label = computed(() => `${props.title} (${props.count ?? 0})`);
</script>

<template>
  <article class="card" v-bind:aria-label="label">
    <h2>{{ label }}</h2>
    <slot />
  </article>
</template>

<style scoped>
.card {
  padding: 1rem;
  border: 1px solid #ddd;
}
</style>
```

`vize fmt --write` produces:

```vue
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ title: string; count?: number }>();
const label = computed(() => `${props.title} (${props.count ?? 0})`);
</script>

<template>
  <article class="card" :aria-label="label">
    <h2>{{ label }}</h2>
    <slot />
  </article>
</template>

<style scoped>
.card {
  padding: 1rem;
  border: 1px solid #ddd;
}
</style>
```

The script is formatted by the Oxc formatter, the same engine Oxfmt uses. The template is re-indented, attribute spacing is normalized, and `v-bind:`, `v-on:` and `v-slot:` become their shorthands. Whitespace inside `<pre>`, `<textarea>` and `v-pre` is left as written.

## Configure the style

In a Vite+ project, set options under `vue.formatter` in `vite.config.ts`:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: {
    printWidth: 100,
    singleQuote: true,
  },
});
```

`fmt.ignorePatterns` applies to both formatters. To hand `.vue` files back to Oxfmt, omit `vue.formatter` and set `fmt.vue: false`. An explicit `vue.formatter` takes precedence over the task control. See [Configuration](./configuration.md) for where the standalone CLI and the editor read the same settings.

The same options are available as CLI flags:

| Flag                               | What it does                                                  |
| ---------------------------------- | ------------------------------------------------------------- |
| `--print-width`                    | Sets the maximum line width.                                  |
| `--tab-width`                      | Sets the number of spaces per indentation level.              |
| `--use-tabs`                       | Indents with tabs instead of spaces.                          |
| `--single-quote`                   | Uses single quotes instead of double quotes.                  |
| `--trailing-comma`                 | Chooses `all`, `es5` or `none` trailing commas.               |
| `--arrow-parens`                   | Chooses `always` or `avoid` for sole arrow parameters.        |
| `--bracket-spacing`                | Toggles spaces inside object literal braces.                  |
| `--bracket-same-line`              | Keeps a multi-line tag's `>` on its final attribute line.     |
| `--no-semi`                        | Omits semicolons at the ends of statements.                   |
| `--sort-attributes`                | Sorts attributes in the template and on block tags.           |
| `--single-attribute-per-line`      | Puts each attribute on its own line.                          |
| `--max-attributes-per-line`        | Wraps after the given number of attributes.                   |
| `--normalize-directive-shorthands` | Turns `v-bind:`, `v-on:` and `v-slot:` into `:`, `@` and `#`. |

Each of these guides covers one setting in detail:

- [Sort imports in Vue scripts](./formatter-import-sorting.md)
- [Indentation width](./formatter-indent-width.md)
- [Line endings](./formatter-line-endings.md)
- [Property quotes](./formatter-property-quotes.md)
- [JSX attribute quotes](./formatter-jsx-quotes.md)

## Options

Every option below can be set under `vue.formatter`. An option you leave out takes its value from `preset`, or else from the default shown here.

| Option                         | Default          | What it does                                                                                                                       |
| ------------------------------ | ---------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `preset`                       | none             | Applies a named set of defaults: `"vize"`, `"prettier"`, `"compact"` or `"documentation"`. Options set next to it win.             |
| `printWidth`                   | `100`            | Maximum line width.                                                                                                                |
| `tabWidth`                     | `2`              | Spaces per indentation level.                                                                                                      |
| `useTabs`                      | `false`          | Indents with tabs.                                                                                                                 |
| `semi`                         | `true`           | Prints semicolons at the ends of statements.                                                                                       |
| `singleQuote`                  | `false`          | Uses single quotes in scripts.                                                                                                     |
| `jsxSingleQuote`               | `false`          | Uses single quotes in JSX attributes.                                                                                              |
| `trailingComma`                | `"all"`          | Where to print trailing commas: `"all"`, `"es5"` or `"none"`.                                                                      |
| `bracketSpacing`               | `true`           | Prints spaces inside object literal braces.                                                                                        |
| `bracketSameLine`              | `false`          | Puts the `>` of a multi-line tag at the end of its last line.                                                                      |
| `arrowParens`                  | `"always"`       | Parentheses around a sole arrow function parameter: `"always"` or `"avoid"`.                                                       |
| `endOfLine`                    | `"lf"`           | Line endings: `"lf"`, `"crlf"`, `"cr"` or `"auto"`. See [Line endings](./formatter-line-endings.md).                               |
| `quoteProps`                   | `"as-needed"`    | When object keys are quoted: `"as-needed"`, `"consistent"` or `"preserve"`. See [Property quotes](./formatter-property-quotes.md). |
| `singleAttributePerLine`       | `false`          | Puts each attribute on its own line.                                                                                               |
| `maxAttributesPerLine`         | none             | Wraps attributes once a tag has more than this many.                                                                               |
| `vueIndentScriptAndStyle`      | `false`          | Indents the contents of `<script>` and `<style>`.                                                                                  |
| `sortAttributes`               | `true`           | Sorts attributes in the template and on block tags.                                                                                |
| `attributeSortOrder`           | `"alphabetical"` | Order within one attribute group: `"alphabetical"` or `"as-written"`.                                                              |
| `mergeBindAndNonBindAttrs`     | `false`          | Sorts `class` and `:class` together instead of static attributes first.                                                            |
| `attributeGroups`              | none             | Custom attribute groups, such as `[["id"], ["v-*"], ["*"]]`, in place of the Vue style guide order.                                |
| `normalizeDirectiveShorthands` | `true`           | Turns `v-bind:`, `v-on:` and `v-slot:` into `:`, `@` and `#`.                                                                      |
| `sortBlocks`                   | `true`           | Orders blocks as script, script setup, template, styles, custom blocks.                                                            |
| `normalizeCssQuotes`           | `false`          | Re-quotes strings in CSS, SCSS and Less with the `singleQuote` preference.                                                         |
| `normalizeInlineStyles`        | `false`          | Writes a one-line static `style` attribute as `color: red; top: 0`.                                                                |
| `normalizeVForParens`          | `false`          | Wraps several `v-for` aliases in parentheses, as in `(item, i) in items`.                                                          |
| `selfCloseVoidElements`        | `false`          | Closes void elements, as in `<img />`.                                                                                             |
| `reindentPreprocessorStyles`   | `false`          | Re-indents SCSS and Less blocks by nesting depth and tidies their blank lines.                                                     |
| `sortImports`                  | off              | Sorts imports in scripts. See [Sort imports](./formatter-import-sorting.md).                                                       |

The native Node `formatSfc` API accepts the five Prettier migration booleans
`normalizeCssQuotes`, `normalizeInlineStyles`, `normalizeVForParens`,
`selfCloseVoidElements` and `reindentPreprocessorStyles` under the same names.
Explicit values override `preset`, including `false`.

Script quote policies are also shared across config, direct CLI overrides and
Node: `--jsx-single-quote[=false]` maps to `jsxSingleQuote`, while
`--quote-props <as-needed|consistent|preserve>` maps to `quoteProps`.

The trailing comma policy is shared in the same way:
`--trailing-comma <all|es5|none>` maps to the typed Node and config option
`trailingComma`. Explicit CLI and Node values override a selected preset.

Arrow parameter parentheses are also projected consistently:
`--arrow-parens <always|avoid>` maps to the typed Node and config option
`arrowParens`, with explicit values taking precedence over presets.

Bracket layout is projected consistently as well:
`--bracket-spacing[=false]` and `--bracket-same-line[=false]` map to the typed
Node and config options `bracketSpacing` and `bracketSameLine`. Explicit true
or false values override a selected preset.

## Compact application code

Use the `compact` preset when application templates should stay dense without reordering authored blocks or attributes:

```ts
export default defineConfig({}).withVue({
  formatter: {
    preset: "compact",
  },
});
```

It sets `printWidth: 120` and `bracketSameLine: true`, preserves authored attribute and SFC block order, and still shortens directive names such as `v-bind:id` to `:id`. It is only a named assignment of existing options, so options written next to it still win. The equivalent CLI option is `vize fmt --preset compact`.

## Documentation examples

Use the `documentation` preset for examples that should stay narrow and easy to scan in prose:

```ts
export default defineConfig({}).withVue({
  formatter: {
    preset: "documentation",
  },
});
```

It sets `printWidth: 80` and `singleAttributePerLine: true`, keeps authored attribute and SFC block order, and still shortens directive names. It does not enable the Prettier migration normalizations for CSS, inline styles, `v-for` aliases, void elements or preprocessor layout. The equivalent CLI option is `vize fmt --preset documentation`.

## Coming from Prettier

Vize is close to Prettier and Oxfmt, but a few defaults differ on purpose. To format Vue files the way Prettier does, set the `prettier` preset:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: {
    preset: "prettier",
  },
});
```

The preset keeps attributes and blocks in the written order (`sortAttributes: false`, `sortBlocks: false`) and also:

- sets the print width to 80, as in Prettier. The Vize default is 100, as in Oxfmt;
- leaves `v-bind:`, `v-on:` and `v-slot:` as written;
- writes strings in CSS, SCSS and Less with double quotes, or with single quotes when you add `--single-quote`;
- writes a one-line `style` attribute as `color: red; top: 0`, without a trailing `;`;
- writes several `v-for` aliases in parentheses, as in `v-for="(item, i) in items"`;
- closes void elements, as in `<img src="a.png" />`;
- re-indents SCSS and Less blocks by nesting depth and tidies their blank lines.

On the command line, `vize fmt --preset prettier` applies the same preset. Options resolve in this order: the preset in the config file, the other options in the config file, `--preset`, then the other flags.

The Node API accepts the same four presets, `vize`, `prettier`, `compact` and `documentation`:

```ts
import { formatSfc } from "@vizejs/native";

const result = formatSfc(source, {
  preset: "prettier",
  singleQuote: true,
});
```

Explicit options override the preset, including `false`. Omitting `preset` keeps the existing Vize defaults; unknown preset names are rejected.

With `sortAttributes: false`, the attributes of `<script>`, `<template>`, `<style>` and custom block tags also stay in the written order. With the default, `<script lang="ts" setup>` becomes `<script setup lang="ts">`.

These differences remain, even with the preset:

- Text inside elements keeps its line breaks. Prettier refills long text up to the print width.
- Template expressions other than `v-for` keep their written form, for example the `,` separators in a slot type such as `{ open: boolean, close: () => void }`. Prettier uses `;`.
- Values in SCSS and Less blocks are kept as written; the preset only changes their quotes, indentation and blank lines. Prettier also reformats values, for example `.2s` to `0.2s`. Indented Sass and Stylus blocks are left as written, and Pug templates only get their outer indentation adjusted.

## Format on save

The [editor setup](./vite-plus-editor.md) selects Vize as the formatter for Vue files and keeps your existing formatter for everything else.
