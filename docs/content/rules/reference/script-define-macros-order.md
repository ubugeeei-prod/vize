---
title: "script/define-macros-order"
description: "Enforce a consistent order of the Vue compiler macros in <script setup>."
---

# `script/define-macros-order`

Enforce a consistent order of the Vue compiler macros in &lt;script setup&gt;.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | [Rule options](../options.md) |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`defineEmits` comes before `defineProps`, but `defineProps` and then `defineEmits` must be the first statements of `<script setup>`.

```vue annotate="remove:2,3"
<script setup lang="ts">
// defineEmits before defineProps
const emit = defineEmits<{ change: [value: string] }>()
const props = defineProps<{ count: number }>()
</script>
```

## Good

`defineProps` and then `defineEmits` come right after the imports, before every other statement; the other macros may follow in any order.

```vue annotate="add:2,4,5,6"
<script setup lang="ts">
import { ref } from 'vue'
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
const model = defineModel<string>()
const value = ref(0)
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/define-macros-order": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L44)
