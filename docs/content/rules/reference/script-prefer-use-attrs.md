---
title: "script/prefer-use-attrs"
---

# `script/prefer-use-attrs`

Recommend using useAttrs() over context.attrs

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
        "script/prefer-use-attrs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`setup` obtains `attrs` by destructuring its context parameter, which this rule asks to replace with the Composition API helper.

```vue
<script lang="ts">
export default { setup(_props, { attrs }) { console.log(attrs.class); } };
</script>
```

## Good

`useAttrs()` supplies `attrs` inside setup, retaining the `attrs.class` read without depending on the second setup parameter.

```vue
<script lang="ts">
import { useAttrs } from "vue";
export default { setup() { const attrs = useAttrs(); console.log(attrs.class); } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44) · [All rules](../all.md)
