---
title: "script/valid-define-options"
---

# `script/valid-define-options`

Enforce valid defineOptions() usage (single object arg, no props/emits/expose/slots)

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
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-options": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The first call puts the dedicated `props` declaration inside `defineOptions`; the later calls also repeat the macro and include a non-object argument. These illustrate the forbidden shape and repeated-call constraints.

```vue
<script setup lang="ts">
defineOptions({ props: ['foo'] })   // use defineProps instead
defineOptions({ name: 'Foo' })
defineOptions({ name: 'Bar' })      // duplicate call
defineOptions('Foo')                // not an object literal
</script>
```

## Good

One `defineOptions` call receives an object containing only the supported ordinary options `name` and `inheritAttrs`.

```vue
<script setup lang="ts">
defineOptions({ name: 'Foo', inheritAttrs: false })
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41) · [All rules](../all.md)
