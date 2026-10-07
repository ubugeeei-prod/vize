---
title: "musea/require-component"
---

# `musea/require-component`

Require component attribute in <art> block

Default severity: `warning`  
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
        "musea/require-component": "warn"
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
<art title="Button">
  <variant name="primary" />
</art>
```

## Good

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11) · [All rules](../all.md)
