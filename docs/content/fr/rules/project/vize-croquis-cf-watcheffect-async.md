---
title: "vize:croquis/cf/watcheffect-async"
---

# `vize:croquis/cf/watcheffect-async`

Protégez le travail asynchrone de watchEffect contre l’invalidation et les réponses périmées.

| Gravité par défaut | Correction automatique | Options |
| --- | --- | --- |
| error | Aucune ; examinez les fichiers concernés et appliquez la correction | Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité |

Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous

## Champ d’application et limites

Le watchEffect asynchrone présenté ici n’a ni nettoyage lors de l’invalidation ni garde contre les réponses périmées. Vue prend en charge le nettoyage de watchEffect ; l’exemple Bon utilise une source de watch explicite et annule le travail invalidé.

## Fichiers communs du projet

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./SearchPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export interface Result { items: string[]; }
export async function load(query: string, options?: { signal?: AbortSignal }): Promise<Result> {
  const response = await fetch(`/search?q=${encodeURIComponent(query)}`, options);
  return response.json();
}
```

## Mauvais

Le watchEffect asynchrone présenté ici n’a ni nettoyage lors de l’invalidation ni garde contre une réponse périmée. Le `watchEffect` asynchrone mêle la collecte implicite des dépendances à une requête dont il attend le résultat, sans garde contre l’invalidation.

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watchEffect } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watchEffect(async () => {
  result.value = await load(props.query);
});
</script>
```

## Bon

Un `watch(() => props.query, ...)` explicite déclare la source, enregistre le nettoyage de la requête et refuse une réponse périmée après invalidation.

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value, _oldValue, onCleanup) => {
    const controller = new AbortController();
    let active = true;

    onCleanup(() => {
      active = false;
      controller.abort();
    });

    const next = await load(value, { signal: controller.signal });
    if (active) result.value = next;
  },
);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/watcheffect-async": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## Voir aussi

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[Index des règles inter-fichiers](../cross-file.md)
