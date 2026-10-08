---
title: "script/define-props-declaration"
---

# `script/define-props-declaration`

Enforce type-based defineProps<{ ... }>() over the runtime/object form

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
        "script/define-props-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`defineProps({ title: String })` supplies a runtime object, which conflicts with this rule’s preference for type-based props.

```vue
<script setup lang="ts">
const props = defineProps({ title: String });
console.log(props.title);
</script>
```

## Good

`defineProps<{ title: string }>()` declares `title` in the type argument and retains the `props.title` access without a runtime declaration argument.

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
console.log(props.title);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40) · [All rules](../all.md)
