---
title: "script/require-explicit-emits"
description: "Require emitted events to be declared in defineEmits or the emits option."
---

# `script/require-explicit-emits`

Require emitted events to be declared in defineEmits or the emits option.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The captured emit function emits `save`, but `defineEmits([])` declares no such event.

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits([]);
emit("save");
</script>
```

## Good

Adding `"save"` to the declaration makes the emitted literal event part of the component’s explicit event contract.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(["save"]);
emit("save");
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-explicit-emits": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L59)
