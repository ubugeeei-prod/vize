---
title: "script/require-slots-as-functions"
---

# `script/require-slots-as-functions`

Require $slots properties to be called as functions

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
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/require-slots-as-functions": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`render` returns `this.$slots.default` without calling it, which was the Vue 2 vnode array.

```vue
<script lang="ts">
export default {
  render() {
    return this.$slots.default
  }
}
</script>
```

## Good

`render` calls `this.$slots.default()` and uses the returned vnode.

```vue
<script lang="ts">
export default {
  render() {
    return this.$slots.default()
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_slots_as_functions.rs#L44) · [All rules](../all.md)
