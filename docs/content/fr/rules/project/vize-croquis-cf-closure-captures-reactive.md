---
title: "vize:croquis/cf/closure-captures-reactive"
---

# `vize:croquis/cf/closure-captures-reactive`

Une fermeture capture une valeur réactive et ne verra pas les mises à jour ultérieures.

| Gravité par défaut | Correction automatique | Options |
| --- | --- | --- |
| Non émis | Aucune ; examinez les fichiers concernés et appliquez la correction | Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité |

Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

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
import { computed, ref } from 'vue';
import { makeReader } from './reader';
const count = ref(0);
const read = makeReader(count);
const shown = computed(read);
</script>

<template>
<button @click="count++">Increment {{ count }}</button><p>{{ shown }}</p>
</template>

```

## Mauvais

`makeReader` copie `count.value` avant de créer la fermeture. Le lecteur calculé renvoie ensuite ce nombre initial sans lire de dépendance réactive.

`reader.ts`

```ts
import type { Ref } from 'vue';
export function makeReader(count: Ref<number>): () => number {
  const captured = count.value;
  return () => captured;
}

```

## Bon

La fermeture lit `count.value` lorsqu’elle est appelée ; l’accesseur calculé peut donc suivre la ref et mettre à jour `shown` après les incréments.

`reader.ts`

```ts
import type { Ref } from 'vue';
export function makeReader(count: Ref<number>): () => number {
  return () => count.value;
}

```

## Configuration (Vite+)

Le CLI n’émet pas ce code actuellement ; aucune configuration de règle du CLI ne permet donc de l’activer ici.

## Voir aussi

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](../cross-file.md)
