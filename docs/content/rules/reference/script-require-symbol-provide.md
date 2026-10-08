---
title: "script/require-symbol-provide"
---

# `script/require-symbol-provide`

Recommend using Symbol as injection key for provide/inject

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
        "script/require-symbol-provide": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`provide` and `inject` use literal string keys such as `'user'` and `'theme'`, which can collide with another provider using the same spelling.

```vue
<script setup lang="ts">
// String keys can collide
provide('user', user)
const user = inject('user')

// Magic strings are error-prone
provide('theme', { dark: true })
</script>
```

## Good

The shared `UserKey` is created with `Symbol` and annotated as `InjectionKey<User>`; both calls pass that key instead of a literal string.

```vue
<script lang="ts">
// Define injection key with Symbol
export const UserKey: InjectionKey<User> = Symbol('user')

// Provide with Symbol
provide(UserKey, user)

// Inject with Symbol
const user = inject(UserKey)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39) · [All rules](../all.md)
