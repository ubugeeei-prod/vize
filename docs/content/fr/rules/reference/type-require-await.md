---
title: "type/require-await"
description: "Interdire une fonction async qui ne contient aucun await."
---

# `type/require-await`

Interdire une fonction async qui ne contient aucun await.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

noAwait est async, mais son corps n’attend aucune valeur.

```vue annotate="remove:2,3"
<script setup lang="ts">
async function noAwait(): Promise<number> {
  return 1;
}
</script>
```

## Bon

withAwait attend Promise.resolve(1) avant de le renvoyer.

```vue annotate="add:2,3"
<script setup lang="ts">
async function withAwait(): Promise<number> {
  return await Promise.resolve(1);
}
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/require-await": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/require_await.rs#L5)
