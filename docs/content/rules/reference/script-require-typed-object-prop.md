---
title: "script/require-typed-object-prop"
---

# `script/require-typed-object-prop`

Require an explicit type on a prop whose runtime type is `Object` or `Array`

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-typed-object-prop": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

Bare `Object` and `Array` constructors describe only broad runtime categories, so neither `user` nor the `items` element shape has an explicit static type.

```vue
<script setup lang="ts">
const props = defineProps({ user: Object, items: { type: Array } });
</script>
```

## Good

`PropType<User>` and `PropType<User[]>` add the object and element types while retaining the same runtime constructors.

```vue
<script setup lang="ts">
import type { PropType } from "vue";
interface User { name: string }
const props = defineProps({
  user: Object as PropType<User>,
  items: { type: Array as PropType<User[]> },
});
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) · [All rules](../all.md)
