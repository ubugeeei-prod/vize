---
title: "script/order-in-components"
description: "Enforce the default order of properties in component options."
---

# `script/order-in-components`

Enforce the default order of properties in component options.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`data` is written before `props`, so a later group appears above an earlier one.

```vue annotate="remove:5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 }
  },
  props: ['label']
}
</script>
```

## Good

`props` comes before `data`, which matches the default group order.

```vue annotate="add:3,6"
<script lang="ts">
export default {
  props: ['label'],
  data() {
    return { count: 0 }
  }
}
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/order-in-components": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/order_in_components.rs#L35)
