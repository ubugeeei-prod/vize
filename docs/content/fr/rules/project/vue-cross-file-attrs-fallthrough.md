---
title: "vue/cross-file-attrs-fallthrough"
---

# `vue/cross-file-attrs-fallthrough`

Un parent transmet des attributs à un enfant résolu dont la racine ne peut pas en hériter et qui n’utilise pas explicitement $attrs.

| Gravité par défaut | Correction automatique | Options |
| --- | --- | --- |
| warning | Aucune | crossFile ; gravité de la règle (off/warn/error) |

Champ d’application: Déclarations du projet accessibles et composants importés

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

Le parent transmet `class="notice"` à un enfant résolu constitué d’un fragment, qui n’a aucune cible automatique pour les attributs et ne lit jamais `$attrs`.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main>Content</main><aside>Help</aside></template>
```

## Bon

L’enfant choisit `<main>` comme cible en y liant `$attrs` ; son frère `<aside>` reste distinct.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "vue/cross-file-attrs-fallthrough": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## Voir aussi

[Index des règles inter-fichiers](../cross-file.md)
