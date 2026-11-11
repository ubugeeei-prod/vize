---
title: "type/no-implied-eval"
description: "Interdire une API d’exécution dynamique à partir d’une chaîne."
---

# `type/no-implied-eval`

Interdire une API d’exécution dynamique à partir d’une chaîne.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

setTimeout reçoit une chaîne, qui est exécutée comme du code dynamique.

```vue annotate="remove:2,3"
<script setup lang="ts">
function logVoid(): void {}
setTimeout("logVoid()", 10);
</script>
```

## Bon

setTimeout reçoit une fonction, donc aucune chaîne n’est évaluée.

```vue annotate="add:2"
<script setup lang="ts">
setTimeout(() => {}, 10);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-implied-eval": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_implied_eval.rs#L5)
