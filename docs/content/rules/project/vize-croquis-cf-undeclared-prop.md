---
title: "vize:croquis/cf/undeclared-prop"
---

# `vize:croquis/cf/undeclared-prop`

A parent passes a prop the child does not declare.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

## Shared project files

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## Bad

The parent passes `typo` even though the resolved child declares only `title`. This analyzer convention is separate from Vue's general fallthrough-attribute behavior.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" :typo="true" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

## Good

Remove the unintended `typo` binding and retain the declared `title` prop.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[Cross-file index](../cross-file.md)
