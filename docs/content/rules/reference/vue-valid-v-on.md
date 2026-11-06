---
title: "vue/valid-v-on"
description: "Enforce valid `v-on` directives."
---

# `vue/valid-v-on`

Enforce valid `v-on` directives.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Modifier validation

Unsupported modifiers are reported individually at the modifier name, including on long-form `v-on:click`, dynamic event arguments, and object listener syntax. The accepted names match `eslint-plugin-vue` 10.9.2: Vue's built-in event modifiers, the complete keyboard alias table, any single Unicode code point, and names with a finite decimal integer prefix (JavaScript `Number.parseInt(name, 10)` semantics). For example, `.13suffix` is accepted by this rule. Numeric-key deprecation and `.native` migration diagnostics belong to separate rules.

Only `.stop` and `.prevent` provide behavior without a handler. A modifier such as `.once`, `.enter`, or a configured custom name still needs a handler expression.

Projects with custom event handling can add names under `vue.linter.ruleOptions`:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/valid-v-on": "error" },
    ruleOptions: { "vue/valid-v-on": { modifiers: ["custom"] } },
  },
});
```

This allows `@click.custom="handler"`. Names are exact and case-sensitive; options supplement the built-in names and do not enable a disabled rule. Scoped config entries can replace inherited names with a new list or reset them with `[]`.

The keyboard table is adapted from the MIT-licensed [eslint-plugin-vue keyboard aliases](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/key-aliases.json). Its source and license are preserved alongside the implementation.

## Bad

The listener forms omit an event argument or their required handler/object expression.

```vue annotate="remove:2,3,4,5,6,7"
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
  <div @click.sync="handleClick"></div>
  <div @click.nativeish="handleClick"></div>
  <div @click.once></div>
</template>
```

## Good

Use an event with its handler, or pass a listener object to argument-free `v-on`.

```vue annotate="add:2,3,4,5,6"
<template>
  <div @click="handleClick"></div>
  <form @submit.prevent></form>
  <input @keyup.page-down="handlePageDown" />
  <input @keyup.あ="handleKey" />
  <div v-on="{ click: handleClick }"></div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-on": "error"
    },
    "ruleOptions": {
      "vue/valid-v-on": {
        "modifiers": [
          "custom"
        ]
      }
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30)
