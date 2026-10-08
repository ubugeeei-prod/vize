---
title: "script/no-deprecated-destroyed-lifecycle"
---

# `script/no-deprecated-destroyed-lifecycle`

Disallow deprecated destroyed and beforeDestroy lifecycle hooks

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: _none_  
Automatic fix: Available for supported findings  
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
        "script/no-deprecated-destroyed-lifecycle": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`beforeDestroy` is the removed Vue 2 lifecycle option used for the timer cleanup.

```vue
<script lang="ts">
export default { beforeDestroy() { clearTimeout(this.timer); } };
</script>
```

## Good

Renaming the hook to `beforeUnmount` preserves the cleanup body under its Vue 3 lifecycle name.

```vue
<script lang="ts">
export default { beforeUnmount() { clearTimeout(this.timer); } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17) · [All rules](../all.md)
