---
title: "script/valid-define-emits"
description: "Enforce valid defineEmits() usage (no type+runtime args, no local references, single call)."
---

# `script/valid-define-emits`

Enforce valid defineEmits() usage (no type+runtime args, no local references, single call).

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The same `defineEmits` call supplies both a type argument and the runtime array `["save"]`, mixing two mutually exclusive declarations.

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits<{ save: [] }>(["save"]);
</script>
```

## Good

Removing the runtime argument leaves a single type-based event declaration for `save`.

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-emits": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45)
