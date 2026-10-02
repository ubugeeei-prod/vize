---
title: 型ルール オプション
---

# 型ルール オプション

## `type/strict-boolean-expressions`

Enable the native type-aware rule explicitly under `linter.rules`. It belongs
to no preset. Use `linter.ruleOptions` to choose which non-boolean conditions
to allow. The native checker projection already uses strict checking.

```json
{
  "linter": {
    "typeAware": true,
    "rules": { "type/strict-boolean-expressions": "error" },
    "ruleOptions": {
      "type/strict-boolean-expressions": {
        "allowString": false,
        "allowNumber": false,
        "allowNullableObject": false
      }
    }
  }
}
```

With these options, the following script and template conditions は悪い例です:

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

良い例: use comparisons that state the intended treatment of zero, empty text
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
