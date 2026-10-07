Same rule, same 0.432.0, for bindings that are not props of the child at all (fallthrough attributes): a string template literal for `:id` and an object literal for `:class` on a component are reported, while the same `:for` binding on the native `<label>` next to it is not.

`src/MyButton.vue`

```vue
<script setup lang="ts">
defineProps<{ color?: string }>();
</script>

<template>
  <button type="button" class="my-button"><slot></slot></button>
</template>
```

`src/MyPanel.vue`

```vue
<script setup lang="ts">
import { useId } from "vue";

import MyButton from "./MyButton.vue";

const emit = defineEmits<{ close: [] }>();
const id = useId();
const compact = true;
</script>

<template>
  <div class="panel">
    <label :for="`${id}-action`">Action</label>
    <MyButton :id="`${id}-action`" :class="{ 'is-compact': compact }" color="primary" @click="() => emit('close')">
      Close
    </MyButton>
  </div>
</template>
```

`vize.config.json`: `{ "linter": { "typeAware": true, "rules": { "type/no-unsafe-template-binding": "error" } } }`

```console
$ npx vize lint -f plain --help-level none src/MyPanel.vue
src/MyPanel.vue:14:20 error type/no-unsafe-template-binding Template binding resolves to an unsafe `any` or `unknown` type      # :id="`${id}-action`"
src/MyPanel.vue:14:44 error type/no-unsafe-template-binding Template binding resolves to an unsafe `any` or `unknown` type      # :class="{ 'is-compact': compact }"
src/MyPanel.vue:14:101 error type/no-unsafe-template-binding Template event handler calls a value with an unsafe `any` or `unknown` type   # () => emit('close')  (#7902)
$ npx vize check
✓ … No type errors found!
```

It looks like the rule checks the expected type at the target (unknown for an undeclared attribute) instead of the type of the expression, which is `string` / `{ 'is-compact': boolean }` here.
