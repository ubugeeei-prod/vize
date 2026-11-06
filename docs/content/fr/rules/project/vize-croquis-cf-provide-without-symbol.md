---
title: "vize:croquis/cf/provide-without-symbol"
---

# `vize:croquis/cf/provide-without-symbol`

`provide` utilise une clé ordinaire au lieu d’un symbole `InjectionKey`.

| Gravité par défaut | Correction automatique | Options |
| --- | --- | --- |
| warning | Aucune ; examinez les fichiers concernés et appliquez la correction | Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité |

Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous

## Fichiers communs du projet

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

## Mauvais

Les deux composants utilisent la chaîne `"theme"` ; des fonctionnalités sans rapport peuvent accidentellement réutiliser cette clé.

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

## Bon

Exporter un seul symbole typé `ThemeKey` et importer cette même valeur aux endroits où provide et inject sont appelés. Créer des symboles distincts avec la même description ne les relierait pas.

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
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

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "incremental", crossFile: true,
    rules: { "croquis/cf/provide-without-symbol": "warn" },
  },
});
```

```sh
vp run lint
```

## Voir aussi

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/keys.rs)

[Index des règles inter-fichiers](../cross-file.md)
