---
title: Prop Rule Options
---

# Prop Rule Options

## `vue/prop-name-casing`

Set `{ casing: "snake_case" }` to require snake_case in the component's own prop
declarations. The default is `"camelCase"`. Runtime arrays, runtime objects, type
literals and Options API declarations use the existing declaration checks;
template attributes belong to `vue/attribute-hyphenation`.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      rules: { "vue/prop-name-casing": "error" },
      ruleOptions: { "vue/prop-name-casing": { casing: "snake_case" } },
    },
  },
});
```

Bad with this config:

```vue
<script setup>
defineProps({ displayName: String });
</script>
```

Good with this config:

```vue
<script setup>
defineProps({ display_name: String });
</script>
```

Options alone do not enable the rule. A later matching entry replaces the full
option object; `{}` selects the default camelCase policy again. Invalid casing
values and unknown fields are rejected. The upstream `ignoreProps` option is not
exposed by this setting.

Renaming a prop changes the component's public API, so this rule does not offer
an automatic fix. Its authored diagnostic range covers the complete declaration.
For migration from [eslint-plugin-vue](https://eslint.vuejs.org/rules/prop-name-casing.html),
replace `["error", "snake_case"]` with the separate `rules` and `ruleOptions`
entries shown above.
