---
title: "script/no-import-compiler-macros"
description: "Disallow importing Vue compiler macros that are auto-imported."
---

# `script/no-import-compiler-macros`

Disallow importing Vue compiler macros that are auto-imported.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The `vue` import includes `defineProps` and `defineEmits`, although these are compiler macros available directly in `<script setup>`.

```vue annotate="remove:2"
<script setup lang="ts">
import { defineProps, defineEmits } from "vue";
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

## Good

Removing the macro imports leaves both typed macro calls intact; no runtime import is needed for either declaration.

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-import-compiler-macros": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39)
