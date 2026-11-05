---
title: "vize:croquis/cf/template-ref-timing"
---

# `vize:croquis/cf/template-ref-timing`

Une référence de template est lue avant le montage du composant.

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

`focus-input.ts`

```ts
export function focusInput(input: HTMLInputElement | null): void { input?.focus(); }

```

## Mauvais

Setup lit la référence de template avant le montage, alors que sa valeur est encore null. L’appel optionnel de focus ne donne donc le focus à aucun élément.

`App.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { focusInput } from './focus-input';
const input = ref<HTMLInputElement | null>(null);
focusInput(input.value);
</script>

<template>
<input ref="input" aria-label="Name" />
</template>

```

## Bon

`onMounted` diffère la lecture jusqu’à ce que Vue ait affecté l’élément de saisie à la référence de template, permettant à l’utilitaire de focus d’agir dessus.

`App.vue`

```vue
<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { focusInput } from './focus-input';
const input = ref<HTMLInputElement | null>(null);
onMounted(() => { focusInput(input.value); });
</script>

<template>
<input ref="input" aria-label="Name" />
</template>

```

## Configuration (Vite+)

Le CLI n’émet pas ce code actuellement ; aucune configuration de règle du CLI ne permet donc de l’activer ici.

## Voir aussi

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](../cross-file.md)
