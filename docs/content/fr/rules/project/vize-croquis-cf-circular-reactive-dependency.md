---
title: "vize:croquis/cf/circular-reactive-dependency"
---

# `vize:croquis/cf/circular-reactive-dependency`

Des calculs réactifs dépendent les uns des autres en formant un cycle.

Gravité par défaut: Selon le contexte  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/circular-reactive-dependency": "warn" },
    },
  },
});
```

```sh
vp run lint
```

Qualification de l’exemple: `illustrative-source-pair`

Le projet Vue complet ci-dessous illustre une boucle de rétroaction de mises à jour et sa correction. Il ne constitue pas un témoin qualifié de diagnostic CLI : le producteur de diagnostics exige des identités de références et des arêtes de flux réactif conservées, comme le montre le graphe joint. Ces sources ne prouvent pas que le traitement actuel du code source émettra ce code précis. Les contrôles dédiés de diagnostics sur les graphes à identifiants suivis restent distincts des contrôles de grammaire du code source.

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
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`count-key.ts`

```ts
import type { InjectionKey, Ref } from 'vue';
export const countKey: InjectionKey<Ref<number>> = Symbol('count');
```

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from 'vue';
import { countKey } from './count-key';
import CycleView from './CycleView.vue';
const count = ref(1); // A: the provider-owned source.
provide(countKey, count);
</script>
<template>
  <button @click="count++">Increment</button>
  <CycleView />
</template>
```

## Mauvais

App possède et fournit count (A). CycleView en dérive nextCount (B), puis réécrit immédiatement chaque valeur dérivée dans le même count injecté. Chaque écriture modifie à nouveau l’entrée du calcul, créant une boucle de mises à jour A → B → A. Les identités du graphe conservé ci-dessous représentent ces deux références, et non des liaisons sans rapport portant les mêmes noms.

`CycleView.vue`

```vue
<script setup lang="ts">
import { computed, inject, watch } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
watch(nextCount, value => { count.value = value; }, { immediate: true });
</script>
<template><p>{{ nextCount }}</p></template>
```

```text
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B; B -> A
```

## Bon

Supprimer l’observateur qui réécrit B dans A. App conserve la propriété de count et ne le modifie que par son action explicite Increment ; CycleView lit la valeur dérivée nextCount sans réinjecter le résultat. Les mêmes références ne conservent que la dépendance A → B.

`CycleView.vue`

```vue
<script setup lang="ts">
import { computed, inject } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
</script>
<template><p>{{ nextCount }}</p></template>
```

```text
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B
```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Index des règles inter-fichiers](../cross-file.md)
