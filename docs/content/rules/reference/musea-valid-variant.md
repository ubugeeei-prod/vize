---
title: "musea/valid-variant"
---

# `musea/valid-variant`

Require name attribute in <variant> blocks

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Musea .art.vue art, variant, and style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/valid-variant": "error"
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
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

## Good

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8) · [All rules](../all.md)
