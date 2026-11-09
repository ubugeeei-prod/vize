---
title: "vize:croquis/cf/destructuring-breaks-reactivity"
---

# `vize:croquis/cf/destructuring-breaks-reactivity`

Destructuring a reactive object copies the fields and drops tracking.

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

Ordinary destructuring of the `props` object copies its current `item` value; this is separate from direct Vue 3.5 `defineProps()` destructuring.

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

`toRef(props, "item")` retains the connection to the property on `props`.

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

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "incremental", crossFile: true,
    rules: { "croquis/cf/destructuring-breaks-reactivity": "warn" },
  },
});
```

```sh
vp run lint
```

## Related

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis.rs)

[Cross-file index](../cross-file.md)
