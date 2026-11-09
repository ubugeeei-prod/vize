---
title: "vue/no-ref-object-reactivity-loss"
description: "Disallow copying a ref value in the same scope."
---

# `vue/no-ref-object-reactivity-loss`

Disallow copying a ref value in the same scope.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

The check follows recognized ref factories and same-scope value copies. Reads inside another function and writes to `.value` are left alone. It is a source-based opt-in check, without a native type-query requirement.

## Bad

Reading `count.value` into `snapshot` at the script root captures the current number rather than a value that follows later ref updates.

```vue annotate="remove:2,4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const snapshot = count.value;
</script>
```

## Good

A computed getter reads `count.value` when its dependency changes, preserving a reactive result instead of a root-scope copy.

```vue annotate="add:2,4"
<script setup lang="ts">
import { computed, ref } from "vue";
const count = ref(0);
const snapshot = computed(() => count.value);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-ref-object-reactivity-loss": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_object_reactivity_loss.rs#L46)
