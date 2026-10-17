---
title: Diagnostics
description: Read a Vize diagnostic, find its rule, and decide whether to fix, configure or suppress it.
---

# Diagnostics

Every problem Vize reports is a diagnostic: a rule ID, a severity, a location and usually a suggested fix. This page shows how to read one, how to change the way diagnostics are printed, and what to do about them.

To look up a specific rule, go straight to [All Rules](../rules/all.md).

## Read a diagnostic

Take a component with two problems:

```vue
<script setup lang="ts">
import { ref } from "vue";

const items = ref([{ id: 1, name: "Ada" }]);
</script>

<template>
  <ul>
    <li v-for="item in items">{{ item.name }}</li>
  </ul>
  <img src="/logo.png" />
</template>
```

`vize lint src/App.vue --help-level short` prints:

```text
  × [vize:vue/require-v-for-key] Elements in iteration expect to have 'v-bind:key' directives. Element: <li>
    ╭─[src/App.vue:9:9]
  8 │   <ul>
  9 │     <li v-for="item in items">{{ item.name }}</li>
    ·         ─────────────────────
 10 │   </ul>
    ╰────
  help: Why: The :key attribute helps Vue's virtual DOM efficiently track and update list items.
        Reference: docs/content/rules/vue.md


  ⚠ [vize:a11y/alt-text] <img> elements must have an alt attribute
    ╭─[src/App.vue:11:3]
 10 │   </ul>
 11 │   <img src="/logo.png" />
    ·   ───────────────────────
 12 │ </template>
    ╰────
  help: Add an alt attribute: <img alt="Description"> or <img alt=""> for decorative images
        Reference: docs/content/rules/accessibility.md


1 error, 1 warning in 1 file
Linted 1 files in 1.8917ms
```

| Part      | In the example               | What it tells you                                                                                                  |
| --------- | ---------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Severity  | `×` and `⚠`                  | `×` is an error and `⚠` is a warning. An error makes the command exit with status 1.                               |
| Rule ID   | `vize:vue/require-v-for-key` | The rule that reported the problem. In configuration, use the part after `vize:`.                                  |
| Location  | `src/App.vue:9:9`            | File, line and column. The underlined source below it is the exact range.                                          |
| Help      | `help: …`                    | Why the rule exists and how to fix the code. Without `--help-level short`, the help includes a corrected example.  |
| Reference | `docs/content/rules/vue.md`  | The rule's category page. The same rule, with Bad and Good examples, is under [Rule reference](../rules/index.md). |

Warnings alone leave the exit status at 0. Pass `--max-warnings 0` to fail when any warning remains.

## Change how diagnostics are printed

| Option         | Values                                                                              | Use it to                                        |
| -------------- | ----------------------------------------------------------------------------------- | ------------------------------------------------ |
| `--format`     | `text` (default), `plain`, `stylish`, `json`, `markdown`, `html`, `agent`, and more | Match a terminal, a CI log, a report, or a tool. |
| `--help-level` | `full` (default), `short`, `none`                                                   | Shorten or hide the help text.                   |
| `--locale`     | `en` (default), `ja`, `zh`                                                          | Read messages and help in another language.      |
| `--quiet`      |                                                                                     | Print only the summary.                          |

`--format plain` keeps one line per diagnostic, which is easier to read in a CI log:

```text
Patina lint report: 1 error, 1 warning in 1 file

src/App.vue
  src/App.vue:9:9 error vue/require-v-for-key Elements in iteration expect to have 'v-bind:key' directives. Element: <li>
    Reference: docs/content/rules/vue.md
    Help:
      Why: The :key attribute helps Vue's virtual DOM efficiently track and update list items.
  src/App.vue:11:3 warning a11y/alt-text <img> elements must have an alt attribute
    Reference: docs/content/rules/accessibility.md
    Help:
      Add an alt attribute: <img alt="Description"> or <img alt=""> for decorative images
```

`--format json` gives a tool the same information as fields: `ruleId`, `severity` (2 for an error, 1 for a warning), `line`, `column`, `endLine`, `endColumn`, `message` and `help`.

## Decide what to do

1. **Fix the code.** Open the rule in the [Rule reference](../rules/index.md) and compare its Bad and Good examples. `vize lint --fix` applies the fixes that are safe to make automatically.
2. **Change the rule for the project.** Set its severity to `"error"`, `"warn"` or `"off"`:

   ```ts
   import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

   export default defineConfig({
     lint: {
       vue: {
         rules: {
           "a11y/alt-text": "error",
           "vue/require-v-for-key": "off",
         },
       },
     },
   });
   ```

   [Rule Options](../rules/options.md) lists the rules that accept more than a severity.

3. **Suppress one occurrence.** Put a comment on the line above it. [Comment Annotations](./comment-annotations.md) covers the other directives.

   ```vue
   <template>
     <ul>
       <!-- @vize:expected -->
       <li v-for="item in items">{{ item.name }}</li>
     </ul>
   </template>
   ```

## Where diagnostics come from

| Kind             | Example ID              | Reported by                                                              |
| ---------------- | ----------------------- | ------------------------------------------------------------------------ |
| Single-file lint | `vue/require-v-for-key` | `vize lint`, the editor, and the Oxlint plugin                           |
| Cross-file       | `vize:croquis/cf/*`     | `vize lint --cross-file`, after Vize builds a graph of the whole project |
| Type-aware       | `type/*`                | `vize lint --type-aware`, using the TypeScript checker                   |
| Musea and CSS    | `musea/*`               | Any lint run that parses Musea art blocks or style content               |

Single-file rules come from Patina, Vize's linter. They can be configured from the Vite+ config, the shared `vue:` section in `vite.config.*`, the CLI, the JavaScript API and the Oxlint plugin.

Cross-file diagnostics compare providers with injectors, track duplicated IDs and spot reactivity hazards across component boundaries. See [Cross-file Complexity](./cross-file-complexity.md) for the reports they produce.

Type-aware diagnostics need the same project configuration that TypeScript sees through `tsconfig.json`, including `compilerOptions.types`, `paths` and project references. Vize does not need a separate `globals` list for those names.

Musea and CSS diagnostics are documented separately because they are not part of the standard Vue template rules.

## Rule Reference

- [Rules overview](../rules/index.md)
- [Vue rules](../rules/vue.md)
- [Accessibility rules](../rules/accessibility.md)
- [Type and script rules](../rules/type-and-script.md)
- [HTML rules](../rules/html.md)
- [SSR rules](../rules/ssr.md)
- [Vapor rules](../rules/vapor.md)
- [Cross-file rules](../rules/cross-file.md)
- [Musea and CSS rules](../rules/musea-and-css.md)
