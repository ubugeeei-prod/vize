---
title: "script/define-props-destructuring"
---

# `script/define-props-destructuring`

Enforce consistent style for defineProps destructuring in &lt;script setup&gt;

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: See [typed options and defaults](../options.md).

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-destructuring": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`defineProps` is assigned to the single `props` binding rather than destructured, contrary to the default destructuring preference.

```vue
<script setup lang="ts">
const props = defineProps<{ foo: string }>()
</script>
```

## Good

The object pattern binds `foo` and `bar` directly and gives the optional `bar` a default. This relies on Vue 3.5+ reactive props destructuring; the configurable `never` mode prefers the opposite form.

```vue
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29) · [All rules](../all.md)
