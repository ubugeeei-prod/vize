---
title: "script/no-async-in-computed"
description: "Interdire les fonctions asynchrones dans les propriétés calculées."
---

# `script/no-async-in-computed`

Interdire les fonctions asynchrones dans les propriétés calculées.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Le getter `computed` est `async` : la récupération produit donc une Promise au lieu d’une valeur calculée dérivée de manière synchrone.

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
import { computed } from "vue";
const data = computed(async () => {
  const response = await fetch("/api/data");
  return response.json();
});
</script>
```

## Bon

La récupération asynchrone est déplacée dans `watch`, qui stocke son résultat dans `data.value`. Le nettoyage annule l’ancienne requête et empêche un callback inactif d’écrire un résultat périmé ; aucun getter calculé asynchrone ne subsiste.

```vue annotate="add:2,3,4,5,6,7,8,9,10,11"
<script setup lang="ts">
import { ref, watch } from "vue";
const query = ref("");
const data = ref<unknown>(null);
watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;
  onCleanup(() => { active = false; controller.abort(); });
  const response = await fetch(`/api/data?q=${encodeURIComponent(value)}`, { signal: controller.signal });
  const next: unknown = await response.json();
  if (active) data.value = next;
});
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-async-in-computed": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_async_in_computed.rs#L49)
