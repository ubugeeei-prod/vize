---
title: "script/consistent-props-emits-key-quotes"
description: "Require consistent quoted keys in props and emits declarations."
---

# `script/consistent-props-emits-key-quotes`

Require consistent quoted keys in props and emits declarations.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The emits declaration mixes the necessarily quoted `update:modelValue` key with the unquoted `close` key.

```vue annotate="remove:4"
<script setup lang="ts">
const emit = defineEmits<{
  "update:modelValue": [value: string]
  close: []
}>()
</script>
```

## Good

Every static key in the declaration is quoted because `update:modelValue` cannot be written as an unquoted JavaScript identifier.

```vue annotate="add:4,6,7,8,9"
<script setup lang="ts">
const emit = defineEmits<{
  "update:modelValue": [value: string]
  "close": []
}>()
const props = defineProps({
  "aria-label": String,
  "disabled": Boolean,
})
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/consistent-props-emits-key-quotes": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/consistent_key_quotes.rs#L49)
