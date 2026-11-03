---
title: "ecosystem/vue-router-unknown-route"
---

# `ecosystem/vue-router-unknown-route`

Le nom est absent du routeur installé complet.

| Gravité par défaut | Correction automatique | Options |
| --- | --- | --- |
| error | Aucune | crossFile ; gravité de la règle (off/warn/error) |

Champ d’application: Déclarations du projet accessibles et composants importés

Le routeur installé complet doit être accessible depuis createApp(...).use(router) dans l’application. Des tables de routes inconnues ou dynamiques ne prouvent pas les diagnostics de noms inconnus. Les paramètres manquants sont des avertissements, car la navigation peut hériter d’une valeur de la route actuelle.

## Fichiers communs du projet

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

## Mauvais

Le routeur installé accessible déclare `user-post`, mais la navigation utilise le nom mal orthographié `user-posts`.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-posts", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

## Bon

Utiliser le nom enregistré `user-post` tout en conservant les deux paramètres de chemin déclarés.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-unknown-route": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## Voir aussi

[Index des règles inter-fichiers](../cross-file.md)
