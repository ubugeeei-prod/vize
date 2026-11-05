---
title: "ssr/no-browser-globals-in-ssr"
description: "Interdire les variables globales propres au navigateur dans un contexte SSR."
---

# `ssr/no-browser-globals-in-ssr`

Interdire les variables globales propres au navigateur dans un contexte SSR.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Setup lit immédiatement `window.innerWidth`, alors que `window` n’existe pas lorsque le composant s’exécute sur le serveur.

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

## Bon

La largeur initiale est une valeur de ref utilisable sur le serveur, et l’accès au navigateur est déplacé dans `onMounted`, qui s’exécute sur le client plutôt que pendant le setup SSR.

```vue annotate="add:2,3,4,5,6"
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "ssr/no-browser-globals-in-ssr": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles SSR](../ssr.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L157)
