---
title: "script/no-options-api"
description: "Disallow Options API patterns in Vapor mode."
---

# `script/no-options-api`

Disallow Options API patterns in Vapor mode.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `opinionated` | No | None |

Applies to: Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts.

## Bad

The default-export object declares Options API `data()`, a component option form prohibited by this rule.

```vue annotate="remove:1,2,3,4,5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

## Good

The component state becomes a Composition API `ref` in Vapor `<script setup>`, removing the Options API object and its `data` option.

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-options-api": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43)
