---
title: "vize:croquis/cf/watcher-outside-setup"
---

# `vize:croquis/cf/watcher-outside-setup`

`watch` ou `watchEffect` est appelé en dehors de `setup`.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Les watchers à la portée du module sont valides lorsque leur propriétaire conserve et appelle une fonction d’arrêt, ou leur donne volontairement la durée de vie de l’application. Cet exemple exige des durées de vie appartenant au composant ; le contrat n’a actuellement aucun producteur de diagnostics.

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
import Observer from './Observer.vue';
</script>

<template>
<Observer /><Observer />
</template>

```

`Observer.vue`

```vue
<script setup lang="ts">
import { useObserver } from './use-observer';
const { count, observed } = useObserver();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ observed }}</p>
</template>

```

## Mauvais

L’observateur est créé au chargement du module, en dehors du setup de chacune des deux instances d’Observer, et les deux instances partagent ses refs. Il n’est pas automatiquement arrêté lorsqu’une instance donnée d’Observer est démontée.

`use-observer.ts`

```ts
import { ref, watch } from 'vue';
const count = ref(0);
const observed = ref(0);
watch(count, next => { observed.value = next; });
export function useObserver() { return { count, observed }; }

```

## Bon

Chaque appel synchrone de setup crée ses propres refs et son propre observateur dans `useObserver`. Vue associe cet observateur à la durée de vie du composant appelant.

`use-observer.ts`

```ts
import { ref, watch } from 'vue';
export function useObserver() {
  const count = ref(0);
  const observed = ref(0);
  watch(count, next => { observed.value = next; });
  return { count, observed };
}

```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](../cross-file.md)
