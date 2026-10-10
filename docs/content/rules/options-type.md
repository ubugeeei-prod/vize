---
title: Type Rule Options
---

# Type Rule Options

Use these options when you want conditionals to express exactly how your app treats zero, empty
strings, and absent objects. `type/strict-boolean-expressions` is opt-in and requires type-aware
linting; setting its options alone does not enable it.

Start with the configuration and Bad/Good examples below, then run `vp run lint` and review the
reported conditions. The [Rule Options guide](./options.md) explains severity, scoped replacement,
and how to configure rules with Vite+ or the standalone CLI.

## `type/strict-boolean-expressions`

Enable the native type-aware rule explicitly under `linter.rules`. It belongs
to no preset. Use `linter.ruleOptions` to choose which non-boolean conditions
to allow. The native checker projection already uses strict checking.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      typeAware: true,
      rules: {
        "type/strict-boolean-expressions": "error",
      },
      ruleOptions: {
        "type/strict-boolean-expressions": {
          allowString: false,
          allowNumber: false,
          allowNullableObject: false,
        },
      },
    },
  },
});
```

With these options, the following script and template conditions are Bad:

```vue
<script setup lang="ts">
defineProps<{ count: number; title: string; element?: HTMLElement }>();
</script>
<template>
  <p v-if="count">Items</p>
  <p v-show="title">Title</p>
  <p v-if="element">Element</p>
</template>
```

Good: use comparisons that state the intended treatment of zero, empty text
and absent objects.

```vue
<template>
  <p v-if="count > 0">Items</p>
  <p v-show="title !== ''">Title</p>
  <p v-if="element != null">Element</p>
</template>
```

Defaults: `allowString`, `allowNumber`, and `allowNullableObject` are `true`.
`allowNullableBoolean`, `allowNullableString`, `allowNullableNumber`,
`allowNullableEnum`, and `allowAny` are `false`. Each field takes a boolean.
Options alone do not enable the rule; later matching entries replace the
whole option object and explicit `off` still wins. Assertion functions,
array predicates and external/Pug templates are outside this rule's scope.
