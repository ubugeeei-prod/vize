---
title: "nuxt/no-nuxt-config-test-key"
description: "Disallow setting `test` key in Nuxt config."
---

# `nuxt/no-nuxt-config-test-key`

Disallow setting `test` key in Nuxt config.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `nuxt` | No | None |

Applies to: Nuxt configuration files (nuxt.config.ts).

## Bad

The exported Nuxt config sets the identifier key `test` to the boolean `true`, the obsolete config shape this rule rejects.

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ test: true });
```

## Good

The empty config removes that boolean `test` property. This example does not forbid a test configuration object.

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({});
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "nuxt/no-nuxt-config-test-key": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Ecosystem and Nuxt rules](../ecosystem.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16)
