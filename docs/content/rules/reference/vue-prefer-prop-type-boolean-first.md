---
title: "vue/prefer-prop-type-boolean-first"
description: "Enforce `Boolean` comes first in component prop types."
---

# `vue/prefer-prop-type-boolean-first`

Enforce `Boolean` comes first in component prop types.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

Changing constructor order can change runtime boolean casting. This opt-in check covers runtime `defineProps` and Options API prop type arrays; it does not reorder TypeScript union types.

## Bad

`String` precedes `Boolean`, so a bare `enabled` attribute uses string-first casting rather than the boolean-first convention.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps({ enabled: [String, Boolean] });
</script>
```

## Good

Place `Boolean` first to give the prop boolean-first casting while retaining its accepted string type.

```vue annotate="add:2"
<script setup lang="ts">
defineProps({ enabled: [Boolean, String] });
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
        "vue/prefer-prop-type-boolean-first": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/prefer_prop_type_boolean_first.rs#L19)
