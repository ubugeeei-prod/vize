---
title: "nuxt/nuxt-config-keys-order"
description: "Prefer recommended order of Nuxt config properties."
---

# `nuxt/nuxt-config-keys-order`

Prefer recommended order of Nuxt config properties.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `nuxt` | Yes | None |

Applies to: Nuxt configuration files (nuxt.config.ts).

## Bad

The config places `ssr` before `modules`, reversing their order in the rule’s recommended Nuxt config key sequence.

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ ssr: true, modules: [] });
```

## Good

Putting `modules` before `ssr` preserves both values while satisfying the prescribed order; the repair changes layout rather than either option’s meaning.

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({ modules: [], ssr: true });
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "nuxt/nuxt-config-keys-order": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Ecosystem and Nuxt rules](../ecosystem.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24)
