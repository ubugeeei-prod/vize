---
title: "type/use-unknown-in-catch-callback-variable"
description: "Exiger une annotation unknown explicite pour la variable du callback catch d’une Promise."
---

# `type/use-unknown-in-catch-callback-variable`

Exiger une annotation unknown explicite pour la variable du callback catch d’une Promise.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

Le callback catch nomme error sans annotation unknown explicite.

```vue annotate="remove:5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
getNumber().catch((error) => {
  console.log(error);
});
</script>
```

## Bon

Le callback catch annote error comme unknown.

```vue annotate="add:5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
getNumber().catch((error: unknown) => {
  console.log(error);
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
      "type/use-unknown-in-catch-callback-variable": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/use_unknown_in_catch_callback_variable.rs#L5)
