---
title: "script/no-reserved-keys"
---

# `script/no-reserved-keys`

Disallow Vue-reserved names as Options API props/data/computed/methods/setup/inject keys

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-reserved-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The returned data key `$el` collides with Vue’s built-in component-instance property and also uses a reserved `$` prefix.

```vue
<script lang="ts">
export default { data() { return { $el: "custom" }; } };
</script>
```

## Good

Renaming the application data to `elementLabel` avoids the built-in instance surface and reserved prefix.

```vue
<script lang="ts">
export default { data() { return { elementLabel: "custom" }; } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L32) · [All rules](../all.md)
