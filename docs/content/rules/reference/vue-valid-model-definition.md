---
title: "vue/valid-model-definition"
description: "Require valid keys in the model option."
---

# `vue/valid-model-definition`

Require valid keys in the model option.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | None | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

This opt-in rule checks Vue 2's `model` option. Only the static keys `prop` and `event` are accepted; their values are not validated. Vue 3 uses the newer component `v-model` contract instead of this option.

## Bad

`events` is not a supported key in Vue 2's component `model` option.

```vue annotate="remove:2"
<script lang="ts">
export default { model: { prop: "value", events: "input" } };
</script>
```

## Good

Use the supported singular `event` key together with `prop` to describe the Vue 2 model contract.

```vue annotate="add:2"
<script lang="ts">
export default { model: { prop: "value", event: "input" } };
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
        "vue/valid-model-definition": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_model_definition.rs#L21)
