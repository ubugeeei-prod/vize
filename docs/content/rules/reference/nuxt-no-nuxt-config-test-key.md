---
title: "nuxt/no-nuxt-config-test-key"
---

# `nuxt/no-nuxt-config-test-key`

Disallow setting `test` key in Nuxt config

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: `nuxt`  
Automatic fix: None; review the suggested change  
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
        "nuxt/no-nuxt-config-test-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The exported Nuxt config sets the identifier key `test` to the boolean `true`, the obsolete config shape this rule rejects.

`nuxt.config.ts`

```ts
export default defineNuxtConfig({ test: true });
```

## Good

The empty config removes that boolean `test` property. This example does not forbid a test configuration object.

`nuxt.config.ts`

```ts
export default defineNuxtConfig({});
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) · [All rules](../all.md)
