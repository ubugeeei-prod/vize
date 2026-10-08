---
title: "script/valid-next-tick"
---

# `script/valid-next-tick`

Require the result of a nextTick() call to be awaited, chained, or given a callback

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/valid-next-tick": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The imported `nextTick()` is a bare expression with no callback, so its returned Promise is ignored and no work waits for the DOM flush.

```vue
<script setup lang="ts">
import { nextTick } from "vue";
nextTick();
</script>
```

## Good

`await nextTick()` consumes the Promise and explicitly waits for the next DOM update before subsequent setup code continues.

```vue
<script setup lang="ts">
import { nextTick } from "vue";
await nextTick();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55) · [All rules](../all.md)
