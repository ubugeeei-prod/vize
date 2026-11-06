---
title: "vue/no-setup-props-reactivity-loss"
description: "Disallow setup prop reads that lose reactivity."
---

# `vue/no-setup-props-reactivity-loss`

Disallow setup prop reads that lose reactivity.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

Checks root-scope prop copies in Options API `setup` and `<script setup>`. Vue 3.5 reactive props destructuring itself is retained; copying a destructured prop later at the script root is reported. Reads inside another function are allowed.

## Bad

Copying `props.count` at the script root keeps its current value and does not follow later prop updates.

```vue annotate="remove:3"
<script setup lang="ts">
const props = defineProps({ count: Number });
const snapshot = props.count;
</script>
```

## Good

Read `props.count` inside a computed getter so the derived result stays connected to prop changes.

```vue annotate="add:2,4"
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps({ count: Number });
const snapshot = computed(() => props.count);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-setup-props-reactivity-loss": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_setup_props_reactivity_loss.rs#L44)
