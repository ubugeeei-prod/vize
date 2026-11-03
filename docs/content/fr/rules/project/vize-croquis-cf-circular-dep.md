---
title: "vize:croquis/cf/circular-dep"
---

# `vize:croquis/cf/circular-dep`

Des composants s’importent mutuellement en formant un cycle.

| Gravité par défaut | Correction automatique | Options |
| --- | --- | --- |
| Non émis | Aucune ; examinez les fichiers concernés et appliquez la correction | Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité |

Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Cela illustre un cycle concret d’initialisation immédiate. Un composant Vue récursif ou un import circulaire n’est pas automatiquement erroné. Aucun producteur actuel n’émet ce code de contrat.

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
import { aLabel } from './a';
</script>

<template>
<p>{{ aLabel }}</p>
</template>

```

`labels.ts`

```ts
export const aPrefix = 'A';
export const bPrefix = 'B';

```

## Mauvais

`a.ts` importe `b.ts`, qui importe à son tour `a.ts`. Tous deux initialisent immédiatement une constante à partir de la constante encore non initialisée de l’autre module, provoquant une erreur de zone morte temporelle.

`a.ts`

```ts
import { bLabel } from './b';
export const aLabel = 'A' + bLabel;

```

`b.ts`

```ts
import { aLabel } from './a';
export const bLabel = 'B' + aLabel;

```

## Bon

Les deux modules lisent des préfixes initialisés dans le module indépendant `labels.ts`, supprimant le cycle et la lecture croisée immédiate.

`a.ts`

```ts
import { bPrefix } from './labels';
export const aLabel = 'A' + bPrefix;

```

`b.ts`

```ts
import { aPrefix } from './labels';
export const bLabel = 'B' + aPrefix;

```

## Configuration (Vite+)

Le CLI n’émet pas ce code actuellement ; aucune configuration de règle du CLI ne permet donc de l’activer ici.

## Voir aussi

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](../cross-file.md)
