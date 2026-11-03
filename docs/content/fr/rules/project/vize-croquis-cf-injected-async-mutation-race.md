---
title: "vize:croquis/cf/injected-async-mutation-race"
---

# `vize:croquis/cf/injected-async-mutation-race`

Une valeur injectée est modifiée par une tâche asynchrone susceptible de provoquer une condition de concurrence.

| Gravité par défaut | Correction automatique | Options |
| --- | --- | --- |
| error | Aucune ; examinez les fichiers concernés et appliquez la correction | Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité |

Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous

## Fichiers communs du projet

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./StoreProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export async function loadCount(query: string, options?: { signal?: AbortSignal }): Promise<number> {
  const response = await fetch(`/count?q=${encodeURIComponent(query)}`, options);
  return Number(await response.text());
}
```

`CountSummary.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { StoreKey } from "./keys/store";
const store = inject(StoreKey)!;
</script>
<template><p>{{ store.count }}</p></template>
```

## Mauvais

`CountLoader.vue` écrit directement le résultat obtenu après attente dans le store injecté partagé avec `CountSummary.vue`, permettant à un travail périmé d’affecter les deux consommateurs.

`keys/store.ts`

```ts
import type { InjectionKey } from "vue";

export interface Store {
  count: number;
}

export const StoreKey: InjectionKey<Store> = Symbol("store");
```

`StoreProvider.vue`

```vue
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);
</script>

<template>
  <CountLoader />
  <CountSummary />
</template>
```

`CountLoader.vue`

```vue
<script setup lang="ts">
import { loadCount } from "./api";
import { inject, ref, watch } from "vue";
import { StoreKey } from "./keys/store";

const store = inject(StoreKey)!;
const query = ref("");

watch(query, async (value) => {
  store.count = await loadCount(value);
});
</script>
```

## Bon

Le chargeur annule le travail invalidé et n’émet qu’un résultat actif. Le fournisseur se charge de modifier le store via `applyLoadedCount`.

`keys/store.ts`

```ts
import type { InjectionKey } from "vue";

export interface Store {
  count: number;
}

export const StoreKey: InjectionKey<Store> = Symbol("store");
```

`StoreProvider.vue`

```vue
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);

function applyLoadedCount(count: number) {
  store.count = count;
}
</script>

<template>
  <CountLoader @loaded="applyLoadedCount" />
  <CountSummary />
</template>
```

`CountLoader.vue`

```vue
<script setup lang="ts">
import { loadCount } from "./api";
import { ref, watch } from "vue";

const emit = defineEmits<{ loaded: [count: number] }>();
const query = ref("");

watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;

  onCleanup(() => {
    active = false;
    controller.abort();
  });

  const count = await loadCount(value, { signal: controller.signal });
  if (active) emit("loaded", count);
});
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/injected-async-mutation-race": "warn" },
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
