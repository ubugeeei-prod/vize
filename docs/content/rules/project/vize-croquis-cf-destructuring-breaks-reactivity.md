---
title: "vize:croquis/cf/destructuring-breaks-reactivity"
---

# `vize:croquis/cf/destructuring-breaks-reactivity`

Destructuring a reactive object copies the fields and drops tracking.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/destructuring-breaks-reactivity": "warn" },
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
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## Bad

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const { item } = props;
</script>
```

## Good

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

Good avoids this finding; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis.rs)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Cross-file index](../cross-file.md)
