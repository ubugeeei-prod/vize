---
title: "vize:croquis/cf/spread-breaks-reactivity"
---

# `vize:croquis/cf/spread-breaks-reactivity`

Spreading a reactive object copies its values and drops the tracking.

| Default severity | Automatic fix | Options |
| --- | --- | --- |
| error | None; review related files and apply the repair | No per-code options; supported CLI findings accept severity overrides |

Applies to: Analyzed component graph and the supported facts described below

## Shared project files

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

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

`UserSummary.vue` spreads `props.user` into a new object, taking a snapshot of the incoming reactive data.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada", role: "admin" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ user: { name: string; role: string } }>();
const copiedUser = { ...props.user };
</script>
```

## Good

`toRef(props, "user")` keeps a reference to the incoming prop instead of copying its fields.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada", role: "admin" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string; role: string } }>();
const user = toRef(props, "user");
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "incremental", crossFile: true,
    rules: { "croquis/cf/spread-breaks-reactivity": "warn" },
  },
});
```

```sh
vp run lint
```

## Related

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Cross-file index](../cross-file.md)
