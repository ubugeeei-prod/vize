---
title: "script/require-render-return"
---

# `script/require-render-return`

Require the component render function to return a value

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
        "script/require-render-return": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The `render` function reads `this.count` inside a branch and never returns a value.

```vue
<script lang="ts">
export default {
  render() {
    if (this.ready) {
      this.count
    }
  }
}
</script>
```

## Good

The `render` function returns `this.count`.

```vue
<script lang="ts">
export default {
  render() {
    return this.count
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_render_return.rs#L35) · [All rules](../all.md)
