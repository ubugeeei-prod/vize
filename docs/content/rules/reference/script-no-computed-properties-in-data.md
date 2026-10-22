---
title: "script/no-computed-properties-in-data"
---

# `script/no-computed-properties-in-data`

Disallow using computed properties inside the component data function

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
        "script/no-computed-properties-in-data": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`data()` returns `this.doubled` while `doubled` is still a computed property, so the read happens before that computed value exists.

```vue
<script lang="ts">
export default {
  data() {
    return { value: this.doubled }
  },
  computed: {
    doubled() { return 1 }
  }
}
</script>
```

## Good

`data()` returns a plain `value`, and `doubled` reads `this.value` only after initialization.

```vue
<script lang="ts">
export default {
  data() {
    return { value: 1 }
  },
  computed: {
    doubled() { return this.value * 2 }
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_computed_properties_in_data.rs#L44) · [All rules](../all.md)
