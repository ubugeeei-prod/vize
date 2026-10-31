---
title: Event listener options
---

# Event listener options

## `vue/v-on-event-hyphenation`

The original `"always"` and `"never"` strings remain valid. The object form defaults to
`{ style: "always", ignore: [], autofix: false }`. Each `ignore` entry is a case-sensitive literal substring,
so `"update:"` exempts `@update:modelValue`, while `"Update:"` and `"update:*"` do not.
Entries must be unique, must contain a non-whitespace character, and must not end in `:exit`.
The whole rule policy is replaced by a later matching configuration; `{}` clears inherited
exemptions and resets `autofix` to `false`. Options do not enable a disabled rule. `ignoreTags` is not exposed.

Set `autofix: true` to offer static event-argument edits, then use the CLI's `--fix` or the
host's fix command to apply them. The edit keeps `@`/`v-on:`, modifiers, and handler expressions.
Names containing `_` keep their diagnostic without a fix. In `never` mode the converter removes
a hyphen before an ASCII word character; a hyphen before a non-ASCII letter can remain and still
be reported. An unchanged edit leaves file bytes untouched.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      rules: { "vue/v-on-event-hyphenation": "warn" },
      ruleOptions: {
        "vue/v-on-event-hyphenation": { style: "always", ignore: ["update:"], autofix: true },
      },
    },
  },
});
```

The [Vue rule options](./options-vue.md#vue-v-on-event-hyphenation) describe the checked listener forms.
