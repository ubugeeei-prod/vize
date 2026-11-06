---
title: "script/prefer-use-template-ref"
description: "Recommend useTemplateRef over ref for template references (Vue 3.5+)."
---

# `script/prefer-use-template-ref`

Recommend useTemplateRef over ref for template references (Vue 3.5+).

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The nullable `input` ref is paired with the template’s literal `ref="input"`, identifying it as an element reference rather than ordinary nullable data.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from 'vue'
const input = ref<HTMLInputElement | null>(null)
</script>
<template>
  <input ref="input" />
</template>
```

## Good

Vue 3.5+ `useTemplateRef<HTMLInputElement>('input')` makes that template reference explicit. The unpaired `error = ref(null)` remains ordinary data and is intentionally outside this rule.

```vue annotate="add:2,3,4,5,6,10"
<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'
// Paired with the template ref below.
const input = useTemplateRef<HTMLInputElement>('input')
// A nullable data ref the template never binds as a ref.
const error = ref(null)
</script>
<template>
  <input ref="input" />
  <p>{{ error }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/prefer-use-template-ref": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75)
