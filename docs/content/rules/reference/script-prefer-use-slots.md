---
title: "script/prefer-use-slots"
description: "Recommend using useSlots() over context.slots."
---

# `script/prefer-use-slots`

Recommend using useSlots() over context.slots.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`setup` destructures `slots` from its context argument, the access form this rule prefers to replace.

```vue annotate="remove:2,4"
<script lang="ts">
import { defineComponent, h } from "vue";
export default defineComponent({
  setup(_props, { slots }) { return () => h("div", slots.default?.()); },
});
</script>
```

## Good

`useSlots()` retrieves the slots inside setup, preserving the render function and its optional default-slot call without a context parameter.

```vue annotate="add:2,4,5,6,7"
<script lang="ts">
import { defineComponent, h, useSlots } from "vue";
export default defineComponent({
  setup() {
    const slots = useSlots();
    return () => h("div", slots.default?.());
  },
});
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/prefer-use-slots": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_slots.rs#L44)
