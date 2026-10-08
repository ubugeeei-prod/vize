---
title: "musea/valid-variant"
---

# `musea/valid-variant`

Require name attribute in &lt;variant&gt; blocks

[Bad](#bad) · [Good](#good)

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

The variant omits the name needed to identify the preview.

```vue
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

## Good

The primary name identifies that variant.

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8) · [All rules](../all.md)
