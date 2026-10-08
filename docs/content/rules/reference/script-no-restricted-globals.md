---
title: "script/no-restricted-globals"
---

# `script/no-restricted-globals`

Disallow references to runtime-environment globals that must go through a typed wrapper

[Bad](#bad) · [Good](#good)

Default severity: `error`  
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
        "script/no-restricted-globals": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The example reads the default restricted globals `process`, `localStorage`, and `sessionStorage` directly, bypassing the project’s explicit config and storage helpers.

```vue
<script setup lang="ts">
const flag = process.env.FEATURE_FLAG
const token = localStorage.getItem('auth.token')
sessionStorage.setItem('view.scroll', String(window.scrollY))
</script>
```

## Good

`useFeatureFlag`, `authStorage.read`, and `viewStorage.write` remove those direct restricted-global references. The remaining `window.scrollY` is not a default restriction of this rule; SSR safety is a separate concern.

```vue
<script setup lang="ts">
// Use a typed config helper that distinguishes server vs. client.
const flag = useFeatureFlag('FEATURE_FLAG')

// Use a typed wrapper that scopes keys and handles SSR / disabled storage.
const token = authStorage.read('auth.token')
viewStorage.write('view.scroll', String(window.scrollY))
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57) · [All rules](../all.md)
