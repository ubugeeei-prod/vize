---
title: "vize:croquis/cf/browser-api-ssr"
---

# `vize:croquis/cf/browser-api-ssr`

Une API réservée au navigateur est utilisée alors que le composant peut être rendu sur le serveur.

| Gravité par défaut | Correction automatique | Options |
| --- | --- | --- |
| warning | Aucune ; examinez les fichiers concernés et appliquez la correction | Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité |

Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous

## Fichiers communs du projet

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

## Mauvais

`window.innerWidth` s’exécute pendant setup, alors qu’un environnement SSR ne dispose pas du `window` du navigateur.

`App.vue`

```vue
<script setup lang="ts">
const width = window.innerWidth;
</script>
<template><p>Content</p></template>
```

## Bon

Initialiser une ref avec une valeur sûre pour le serveur et lire `window` dans `onMounted`, qui s’exécute après le montage côté client.

`App.vue`

```vue
<script setup lang="ts">
import { onMounted, ref } from "vue";
const width = ref(0);
onMounted(() => { width.value = window.innerWidth; });
</script>
<template><p>Content</p></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "incremental", crossFile: true,
    rules: { "croquis/cf/browser-api-ssr": "warn" },
  },
});
```

```sh
vp run lint
```

## Voir aussi

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[Index des règles inter-fichiers](../cross-file.md)
