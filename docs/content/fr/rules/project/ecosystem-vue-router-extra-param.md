---
title: "ecosystem/vue-router-extra-param"
---

# `ecosystem/vue-router-extra-param`

La route ne déclare pas tab ; Vue Router l’ignore.

Gravité par défaut: error  
Champ d’application: Déclarations du projet accessibles et composants importés  
Options: crossFile ; gravité de la règle (off/warn/error)  
Correction automatique: Aucune

Le routeur installé complet doit être accessible depuis createApp(...).use(router) dans l’application. Des tables de routes inconnues ou dynamiques ne prouvent pas les diagnostics de noms inconnus. Les paramètres manquants sont des avertissements, car la navigation peut hériter d’une valeur de la route actuelle.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-extra-param": "warn" },
    },
  },
});
```

```sh
vp run lint
```

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

Le chemin `user-post` déclare `userId` et `postId`, mais la navigation fournit également `tab`, non déclaré, comme paramètre de chemin.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2", tab: "a" } });
</script>
<template><p>Post</p></template>
```

## Bon

Supprimer `tab` de params et conserver uniquement les clés présentes dans le chemin de la route. Utiliser query séparément si l’application a besoin de sélectionner un onglet.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Index des règles inter-fichiers](../cross-file.md)
