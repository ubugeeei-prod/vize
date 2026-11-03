---
title: "nuxt/prefer-import-meta"
description: "Prefer using `import.meta.*` over `process.*`."
---

# `nuxt/prefer-import-meta`

Prefer using `import.meta.*` over `process.*`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `nuxt` | Yes | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`process.client` uses a legacy Nuxt environment flag that the rule asks to migrate to `import.meta`.

```vue annotate="remove:2"
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

## Good

`import.meta.client` keeps the browser-only branch explicit using the replacement environment flag.

```vue annotate="add:2"
<script setup lang="ts">
if (import.meta.client) console.log("browser");
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "nuxt/prefer-import-meta": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Ecosystem and Nuxt rules](../ecosystem.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18)
