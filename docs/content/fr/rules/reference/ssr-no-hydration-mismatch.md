---
title: "ssr/no-hydration-mismatch"
description: "Interdire les valeurs non déterministes qui causent des divergences d’hydratation."
---

# `ssr/no-hydration-mismatch`

Interdire les valeurs non déterministes qui causent des divergences d’hydratation.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le template évalue `Math.random()` pendant le rendu ; le serveur et le client peuvent donc produire des textes différents pour le même paragraphe.

```vue annotate="remove:2"
<template>
  <p>{{ Math.random() }}</p>
</template>
```

## Bon

Le paragraphe affiche l’état stable `seed` au lieu d’un nouveau résultat aléatoire. Dans cet exemple de style Nuxt, `useState` fournit l’état partagé et la valeur d’initialisation est la constante `"stable"`.

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "ssr/no-hydration-mismatch": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles SSR](../ssr.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122)
