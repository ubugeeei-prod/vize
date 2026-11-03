---
title: "vize:croquis/cf/composable-outside-setup"
---

# `vize:croquis/cf/composable-outside-setup`

Un composable est appelé en dehors de `setup`.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Le problème concerne ce composable dépendant du cycle de vie ; il ne s’agit pas d’une interdiction générale des fonctions utilitaires ordinaires ou de tous les appels à la Composition API en dehors du setup. Ce contrat n’a actuellement aucun producteur de diagnostics.

## Fichiers communs du projet

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

`App.vue`

```vue
<script setup lang="ts">
import { useTitle } from './use-title';
const title = useTitle();
</script>

<template>
<h1>{{ title }}</h1>
</template>

```

## Mauvais

L’import de `use-title.ts` enregistre `onMounted` avant qu’un setup de composant soit actif. Appeler sa fonction exportée plus tard ne fait que renvoyer cette ref au niveau du module ; cela ne peut pas rétablir l’appartenance manquée au cycle de vie.

`use-title.ts`

```ts
import { onMounted, ref } from 'vue';
const title = ref('Before mount');
onMounted(() => { title.value = 'Mounted'; });
export function useTitle() { return title; }

```

## Bon

La création de l’état et l’enregistrement du hook sont tous deux déplacés dans `useTitle`, qu’App appelle de manière synchrone dans setup. Le hook de montage appartient désormais à cette instance d’App.

`use-title.ts`

```ts
import { onMounted, ref } from 'vue';
export function useTitle() {
  const title = ref('Before mount');
  onMounted(() => { title.value = 'Mounted'; });
  return title;
}

```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](../cross-file.md)
