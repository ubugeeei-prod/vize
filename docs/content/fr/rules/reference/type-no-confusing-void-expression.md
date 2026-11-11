---
title: "type/no-confusing-void-expression"
description: "Interdire une expression de type void là où une valeur est attendue."
---

# `type/no-confusing-void-expression`

Interdire une expression de type void là où une valeur est attendue.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

logVoid() renvoie void, mais l’initialiseur traite cet appel comme une valeur.

```vue annotate="remove:3"
<script setup lang="ts">
function logVoid(): void {}
const value = logVoid();
</script>
```

## Bon

Appeler logVoid() comme instruction n’utilise le résultat void que pour son effet.

```vue annotate="add:3"
<script setup lang="ts">
function logVoid(): void {}
logVoid();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-confusing-void-expression": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_confusing_void_expression.rs#L5)
