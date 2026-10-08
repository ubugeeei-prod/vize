---
title: "script/prefer-define-options"
---

# `script/prefer-define-options`

Prefer defineOptions() over a plain &lt;script&gt; that only sets name/inheritAttrs

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
        "script/prefer-define-options": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The plain script’s only meaningful statement exports an object containing just `name` and `inheritAttrs`; these options can be expressed by `defineOptions`.

```vue
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

## Good

The shown `data()` method makes the script carry real Options API logic, so it falls outside this rule’s conservative options-only suggestion. This Good demonstrates an allowed exception; the direct migration would put `defineOptions({ name: 'MyComponent', inheritAttrs: false })` in `<script setup>`.

```vue
<script lang="ts">
// Real options logic — keep the plain script.
export default {
name: 'MyComponent',
data() { return { count: 0 } },
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52) · [All rules](../all.md)
