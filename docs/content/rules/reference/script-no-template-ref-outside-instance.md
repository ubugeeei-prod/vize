---
title: "script/no-template-ref-outside-instance"
description: "Disallow useTemplateRef() where no component instance runs it."
---

# `script/no-template-ref-outside-instance`

Disallow useTemplateRef() where no component instance runs it.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`focus()` runs only as the button's click handler, and the `setTimeout` callback runs after setup has returned. Neither has a current component instance, so the refs they create are never set.

```vue annotate="remove:2,4,6"
<script setup lang="ts">
import { useTemplateRef } from 'vue'
function focus() {
  useTemplateRef<HTMLInputElement>('input').value?.focus()
}
setTimeout(() => useTemplateRef('input'))
</script>
<template>
  <input ref="input" />
  <button @click="focus">Focus</button>
</template>
```

## Good

`useTemplateRef('input')` runs once in setup. The click handler and the `onMounted` callback only read `input.value`.

```vue annotate="add:2,3,5,7"
<script setup lang="ts">
import { onMounted, useTemplateRef } from 'vue'
const input = useTemplateRef<HTMLInputElement>('input')
function focus() {
  input.value?.focus()
}
onMounted(() => focus())
</script>
<template>
  <input ref="input" />
  <button @click="focus">Focus</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-template-ref-outside-instance": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_template_ref_outside_instance.rs#L78)
