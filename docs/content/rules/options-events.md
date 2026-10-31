---
title: Event listener options
---

# Event listener options

## `vue/v-on-event-hyphenation`

The original `"always"` and `"never"` strings remain valid. The object form defaults to
`{ style: "always", ignore: [] }`. Each `ignore` entry is a case-sensitive literal substring,
so `"update:"` exempts `@update:modelValue`, while `"Update:"` and `"update:*"` do not.
Entries must be unique, must contain a non-whitespace character, and must not end in `:exit`.
The whole rule policy is replaced by a later matching configuration; `{}` clears inherited
exemptions. Options do not enable a disabled rule. Autofix and `ignoreTags` are not exposed.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      rules: { "vue/v-on-event-hyphenation": "warn" },
      ruleOptions: {
        "vue/v-on-event-hyphenation": { style: "always", ignore: ["update:"] },
      },
    },
  },
});
```

The [Vue rule options](./options-vue.md#vue-v-on-event-hyphenation) describe the checked listener forms.
