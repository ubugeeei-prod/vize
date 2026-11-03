---
title: "vize:croquis/cf/module-scope-reactive"
---

# `vize:croquis/cf/module-scope-reactive`

Un état réactif est créé au niveau du module et partagé par tous les appelants.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Un état réactif à la portée du module est autorisé pour des stores applicatifs conçus à cet effet. Cet exemple suppose une isolation par composant ou requête ; le contrat publié n’a actuellement aucun producteur de diagnostics.

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
import Counter from './Counter.vue';
</script>

<template>
<Counter /><Counter />
</template>

```

`Counter.vue`

```vue
<script setup lang="ts">
import { createCounter } from './counter';
const { count } = createCounter();
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

## Mauvais

Le module initialise `count` une seule fois et les deux instances de Counter reçoivent la même ref. Cliquer sur l’une modifie les deux compteurs, alors que cet exemple prévoit un état indépendant pour chaque instance.

`counter.ts`

```ts
import { ref } from 'vue';
const count = ref(0);
export function createCounter() { return { count }; }

```

## Bon

Créer la ref dans `createCounter` donne à chaque appel synchrone de setup un objet d’état distinct ; chaque bouton possède donc son propre compteur.

`counter.ts`

```ts
import { ref } from 'vue';
export function createCounter() {
  const count = ref(0);
  return { count };
}

```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](../cross-file.md)
