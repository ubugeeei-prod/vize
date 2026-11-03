---
title: "html/cross-component-nesting"
---

# `html/cross-component-nesting`

Vérifier l’imbrication HTML réelle après la composition des composants importés.

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

Le `<p>` du parent contient un enfant résolu dont la racine est `<div>`, ce qui produit une imbrication paragraphe/bloc invalide après composition.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><p><Child /></p></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

## Bon

Utiliser un conteneur `<section>` pouvant contenir l’élément de bloc de l’enfant ; l’enfant reste inchangé.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><section><Child /></section></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "html/cross-component-nesting": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## Voir aussi

[Index des règles inter-fichiers](../cross-file.md)
