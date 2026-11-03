---
title: "script/prefer-use-attrs"
description: "Recommend using useAttrs() over context.attrs."
---

# `script/prefer-use-attrs`

Recommend using useAttrs() over context.attrs.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`setup` obtains `attrs` by destructuring its context parameter, which this rule asks to replace with the Composition API helper.

```vue annotate="remove:2"
<script lang="ts">
export default { setup(_props, { attrs }) { console.log(attrs.class); } };
</script>
```

## Good

`useAttrs()` supplies `attrs` inside setup, retaining the `attrs.class` read without depending on the second setup parameter.

```vue annotate="add:2,3"
<script lang="ts">
import { useAttrs } from "vue";
export default { setup() { const attrs = useAttrs(); console.log(attrs.class); } };
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
        "script/prefer-use-attrs": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44)
