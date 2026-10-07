---
title: "vize:croquis/cf/inject-without-symbol"
---

# `vize:croquis/cf/inject-without-symbol`

`inject` uses a plain key instead of an `InjectionKey` symbol.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/inject-without-symbol": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## Shared project files

Use these unchanged files in both Bad and Good. Install Vue (and vue-router for Router examples) in the project. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ThemeProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";
export interface Theme { color: string; }
export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

## Bad

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";

const theme = ref({ color: "blue" });
provide("theme", theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";

const theme = inject("theme");
</script>
```

## Good

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = ref({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

Good avoids this finding; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/keys.rs)

[Cross-file index](../cross-file.md)
