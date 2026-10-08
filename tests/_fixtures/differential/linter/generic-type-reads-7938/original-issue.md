## Summary

`vue/no-unused-setup-bindings` reports an import as "never read" when it is only referenced from the `generic` attribute of `<script setup>`. The binding is used: removing the import breaks the component's type parameter.

## Reproduction

`src/kind.ts`:

```ts
export enum Kind {
  A = "a",
  B = "b",
}
```

`src/kind-picker.vue`:

```vue
<script setup lang="ts" generic="T extends Kind.A | Kind.B">
import { Kind } from "./kind";

const { value } = defineProps<{ value: T }>();
</script>

<template>
  <span>{{ value }}</span>
</template>
```

`vize.config.json`:

```json
{ "linter": { "rules": { "vue/no-unused-setup-bindings": "warn" } } }
```

```sh
vize lint -c vize.config.json src/kind-picker.vue
```

## Actual (0.432.0)

```text
src/kind-picker.vue:2:10 [vize:vue/no-unused-setup-bindings] Setup binding 'Kind' is never read
```

## Expected

No diagnostic. References inside `generic="..."` should count as uses of the setup bindings (at least as type uses, the same way a reference in a `defineProps<...>()` type argument does).

## Environment

- vize / @vizejs/native 0.432.0
- vue 3.5.38
- macOS arm64, Node 26
