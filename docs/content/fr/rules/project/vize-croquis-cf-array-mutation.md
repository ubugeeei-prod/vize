---
title: "vize:croquis/cf/array-mutation"
---

# `vize:croquis/cf/array-mutation`

Utilisez des mutations observables pour mettre à jour les indices d’un tableau dans Vue 2.7.

| Gravité par défaut | Correction automatique | Options |
| --- | --- | --- |
| Non émis | Aucune ; examinez les fichiers concernés et appliquez la correction | Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité |

Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Uniquement pour l’ancien Vue 2.7 : utilisez des dépendances Vue 2.7 et de compilateur SFC correspondantes pour ce scénario. Les proxies de Vue 3 suivent les affectations aux indices des tableaux ; `items[0] = next` est donc réactif en Vue 3 et ne constitue pas un défaut de Vue 3. Ce code publié n’a actuellement aucun producteur de diagnostics.

## Champ d’application et limites

Ce contrat sans producteur actuel utilise un scénario historique d’affectation à un indice de tableau dans Vue 2.7. Vue 2 ne notifie pas les mises à jour par affectation directe à un indice ; Vue 3 suit ces affectations. Les fichiers Mauvais et Bon illustrent la correction pour Vue 2, et non un défaut de Vue 3.

## Fichiers communs du projet

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

`main.ts`

```ts
import Vue from 'vue';
import App from './App.vue';
new Vue({ render: h => h(App) }).$mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script lang="ts">
import Vue from 'vue';
import { replaceFirst } from './replace-first';
export default Vue.extend({
  data() { return { items: ['Before'] }; },
  methods: { replace() { replaceFirst(this.items, 'After'); } },
});
</script>
<template><section><p>{{ items[0] }}</p><button @click="replace">Replace</button></section></template>

```

## Mauvais

Dans ce projet historique Vue 2.7, `items[0] = next` modifie le tableau sans avertir l’observateur de tableaux de Vue 2 ; le premier élément affiché peut donc ne pas être mis à jour.

`replace-first.ts`

```ts
export function replaceFirst(items: string[], next: string): void {
  items[0] = next;
}

```

## Bon

`splice(0, 1, next)` utilise la méthode de modification de tableau observée par Vue 2, permettant au même remplacement de mettre à jour la vue.

`replace-first.ts`

```ts
export function replaceFirst(items: string[], next: string): void {
  items.splice(0, 1, next);
}

```

## Configuration (Vite+)

Le CLI n’émet pas ce code actuellement ; aucune configuration de règle du CLI ne permet donc de l’activer ici.

## Voir aussi

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](../cross-file.md)
