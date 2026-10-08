---
title: "vize:croquis/cf/di-outside-setup"
---

# `vize:croquis/cf/di-outside-setup`

`provide` or `inject` is called outside `setup`.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

This example uses component provide/inject. `app.provide` and supported `app.runWithContext` injection are different valid ownership surfaces, not prohibited by this scenario. No current producer emits this contract code.

## Shared project files

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`theme.ts`

```ts
import { inject, provide } from 'vue';
import type { InjectionKey } from 'vue';
export const ThemeKey: InjectionKey<string> = Symbol('theme');
export function provideTheme() { provide(ThemeKey, 'dark'); }
export function useTheme() { return inject(ThemeKey, 'light'); }

```

`ThemedText.vue`

```vue
<script setup lang="ts">
import { useTheme } from './theme';
const theme = useTheme();
</script>

<template>
<p>{{ theme }}</p>
</template>

```

## Bad

`main.ts` calls component `provide` with no active component instance. The child’s `inject` therefore cannot receive this intended ancestor value and uses `light`.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
import { provideTheme } from './theme';
provideTheme();
createApp(App).mount('#app');

```

`App.vue`

```vue
<script setup lang="ts">
import ThemedText from './ThemedText.vue';
</script>

<template>
<ThemedText />
</template>

```

## Good

App calls the provider from its setup before rendering the child. The child now inherits the `dark` value from its component ancestor.

`App.vue`

```vue
<script setup lang="ts">
import ThemedText from './ThemedText.vue';
import { provideTheme } from './theme';
provideTheme();
</script>

<template>
<ThemedText />
</template>

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](../cross-file.md)
