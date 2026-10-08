---
title: "script/no-deprecated-dollar-listeners-api"
---

# `script/no-deprecated-dollar-listeners-api`

Disallow the $listeners instance property removed in Vue 3 (merged into $attrs)

[Bad](#bad) · [Good](#good)

Default severity: `error`  
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
        "script/no-deprecated-dollar-listeners-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The member reads and the bare argument reference all use `$listeners`, which Vue 3 removed after merging listeners into attributes.

```vue
<script setup lang="ts">
const handlers = this.$listeners
const forwarded = ctx.$listeners
emit('input', $listeners)
</script>
```

## Good

The reads move to `this.$attrs` and setup-context `ctx.attrs`. These replace the removed listener surface; the illustrated receivers must exist in the surrounding component context.

```vue
<script setup lang="ts">
const handlers = this.$attrs
const forwarded = ctx.attrs
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L40) · [All rules](../all.md)
