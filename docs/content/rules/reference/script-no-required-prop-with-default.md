---
title: "script/no-required-prop-with-default"
---

# `script/no-required-prop-with-default`

Disallow a prop that is both required: true and has a default

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
        "script/no-required-prop-with-default": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`title` is both required and given the fallback `"Untitled"`, combining a required-input contract with a default intended for missing input.

```vue
<script lang="ts">
export default { props: { title: { type: String, required: true, default: "Untitled" } } };
</script>
```

## Good

Removing `required: true` makes `title` optional and leaves `"Untitled"` as its coherent fallback.

```vue
<script lang="ts">
export default { props: { title: { type: String, default: "Untitled" } } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L29) · [All rules](../all.md)
