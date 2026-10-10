---
title: "SSR rules"
---

# SSR rules

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`ssr/no-browser-globals-in-ssr`](#ssr-no-browser-globals-in-ssr) | [Bad](#ssr-no-browser-globals-in-ssr-bad) · [Good](#ssr-no-browser-globals-in-ssr-good) | Disallow browser-only globals in SSR context |
| [`ssr/no-hydration-mismatch`](#ssr-no-hydration-mismatch) | [Bad](#ssr-no-hydration-mismatch-bad) · [Good](#ssr-no-hydration-mismatch-good) | Disallow non-deterministic values that cause hydration mismatch |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `ssr/no-browser-globals-in-ssr`

Disallow browser-only globals in SSR context

[Bad](#ssr-no-browser-globals-in-ssr-bad) · [Good](#ssr-no-browser-globals-in-ssr-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ssr/no-browser-globals-in-ssr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-browser-globals-in-ssr-bad"></span>

**Bad**

Setup reads `window.innerWidth` immediately, although `window` does not exist when the component runs on the server.

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

<span id="ssr-no-browser-globals-in-ssr-good"></span>

**Good**

The initial width is a server-safe ref value, and the browser access moves into `onMounted`, which runs on the client rather than during SSR setup.

```vue annotate="add:2,3,4,5,6"
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L158) · [All rules](all.md)

### `ssr/no-hydration-mismatch`

Disallow non-deterministic values that cause hydration mismatch

[Bad](#ssr-no-hydration-mismatch-bad) · [Good](#ssr-no-hydration-mismatch-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ssr/no-hydration-mismatch": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-hydration-mismatch-bad"></span>

**Bad**

The template evaluates `Math.random()` during rendering, so the server and client can produce different text for the same paragraph.

```vue annotate="remove:2"
<template>
  <p>{{ Math.random() }}</p>
</template>
```

<span id="ssr-no-hydration-mismatch-good"></span>

**Good**

The paragraph renders the stable `seed` state instead of a fresh random result. In this Nuxt-style example, `useState` supplies the shared state and the initializer is the constant `"stable"`.

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122) · [All rules](all.md)
