---
title: "nuxt/nuxt-config-keys-order"
---

# `nuxt/nuxt-config-keys-order`

Prefer recommended order of Nuxt config properties

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: `nuxt`  
Automatic fix: Available for supported findings  
Applies to: Nuxt configuration files (nuxt.config.ts)  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/nuxt-config-keys-order": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The config places `ssr` before `modules`, reversing their order in the rule’s recommended Nuxt config key sequence.

`nuxt.config.ts`

```ts
export default defineNuxtConfig({ ssr: true, modules: [] });
```

## Good

Putting `modules` before `ssr` preserves both values while satisfying the prescribed order; the repair changes layout rather than either option’s meaning.

`nuxt.config.ts`

```ts
export default defineNuxtConfig({ modules: [], ssr: true });
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) · [All rules](../all.md)
