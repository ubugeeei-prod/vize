---
title: "script/valid-define-props"
description: "Enforce valid defineProps() usage (single call, not both type and runtime args, no local references)."
---

# `script/valid-define-props`

Enforce valid defineProps() usage (single call, not both type and runtime args, no local references).

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The same `defineProps` call supplies both `{ title: string }` as a type argument and `{ title: String }` as a runtime argument, which the compiler does not permit together.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ title: string }>({ title: String });
</script>
```

## Good

Removing the runtime object leaves one type-based declaration for `title` instead of combining both declaration forms.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/valid-define-props": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L46)
