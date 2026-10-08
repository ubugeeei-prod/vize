---
title: "type/no-reactivity-loss"
---

# `type/no-reactivity-loss`

Disallow plain snapshots of reactive values across assignments and calls

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: No rule-specific options. Severity and preset selection are configurable.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-reactivity-loss": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

## Bad

`const count = state.count` takes a plain numeric snapshot of the reactive property, so later updates of `state.count` are not reflected in that binding.

```vue
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0 });
const count = state.count;
</script>
```

## Good

`toRef(state, "count")` keeps `count` linked to the original reactive property rather than copying its current primitive value.

```vue
<script setup lang="ts">
import { reactive, toRef } from "vue";
const state = reactive({ count: 0 });
const count = toRef(state, "count");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_reactivity_loss.rs#L12) · [All rules](../all.md)
