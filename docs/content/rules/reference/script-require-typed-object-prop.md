---
title: "script/require-typed-object-prop"
description: "Require an explicit type on a prop whose runtime type is `Object` or `Array`."
---

# `script/require-typed-object-prop`

Require an explicit type on a prop whose runtime type is `Object` or `Array`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

Bare `Object` and `Array` constructors describe only broad runtime categories, so neither `user` nor the `items` element shape has an explicit static type.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ user: Object, items: { type: Array } });
</script>
```

## Good

`PropType<User>` and `PropType<User[]>` add the object and element types while retaining the same runtime constructors.

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
import type { PropType } from "vue";
interface User { name: string }
const props = defineProps({
  user: Object as PropType<User>,
  items: { type: Array as PropType<User[]> },
});
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
        "script/require-typed-object-prop": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63)
