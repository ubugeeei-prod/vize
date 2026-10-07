---
title: "script/require-typed-object-prop"
---

# `script/require-typed-object-prop`

Require an explicit type on a prop whose runtime type is `Object` or `Array`

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

```vue
<script setup lang="ts">
const props = defineProps({
foo: Object,
bar: { type: Array },
})
</script>
```

## Good

```vue
<script setup lang="ts">
const props = defineProps({
foo: Object as PropType<Foo>,
bar: { type: Array as PropType<Bar[]> },
})

// Type-based form carries the element type directly.
const typed = defineProps<{ foo: Foo; bar: Bar[] }>()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) · [All rules](../all.md)
