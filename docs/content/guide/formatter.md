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

In a Vite+ project, set options under `fmt.vize` in `vite.config.ts`:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  fmt: {
    vize: {
      printWidth: 100,
      singleQuote: true,
    },
  },
});
```

`fmt.ignorePatterns` applies to both formatters. Set `fmt.vize: false` to hand `.vue` files back to Oxfmt. See [Configuration](./configuration.md) for where the standalone CLI and the editor read the same settings.

The same options are available as CLI flags:

| Flag                               | What it does                                                  |
| ---------------------------------- | ------------------------------------------------------------- |
| `--print-width`                    | Sets the maximum line width.                                  |
| `--tab-width`                      | Sets the number of spaces per indentation level.              |
| `--use-tabs`                       | Indents with tabs instead of spaces.                          |
| `--single-quote`                   | Uses single quotes instead of double quotes.                  |
| `--no-semi`                        | Omits semicolons at the ends of statements.                   |
| `--sort-attributes`                | Sorts attributes in the template.                             |
| `--single-attribute-per-line`      | Puts each attribute on its own line.                          |
| `--max-attributes-per-line`        | Wraps after the given number of attributes.                   |
| `--normalize-directive-shorthands` | Turns `v-bind:`, `v-on:` and `v-slot:` into `:`, `@` and `#`. |

Each of these guides covers one setting in detail:

- [Sort imports in Vue scripts](./formatter-import-sorting.md)
- [Indentation width](./formatter-indent-width.md)
- [Line endings](./formatter-line-endings.md)
- [Property quotes](./formatter-property-quotes.md)
- [JSX attribute quotes](./formatter-jsx-quotes.md)

## Format on save

The [editor setup](./vite-plus-editor.md) selects Vize as the formatter for Vue files and keeps your existing formatter for everything else.
