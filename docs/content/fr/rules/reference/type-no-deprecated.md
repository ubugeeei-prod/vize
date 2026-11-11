---
title: "type/no-deprecated"
description: "Interdire l’usage d’une déclaration obsolète."
---

# `type/no-deprecated`

Interdire l’usage d’une déclaration obsolète.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

legacy est marquée @deprecated, et l’appel utilise encore cette déclaration.

```vue annotate="remove:2,3,6"
<script setup lang="ts">
/** @deprecated use current() */
function legacy(): number {
  return 1;
}
const legacyValue = legacy();
</script>
```

## Bon

current n’a pas de marque d’obsolescence, donc l’appel utilise une déclaration prise en charge.

```vue annotate="add:2,5"
<script setup lang="ts">
function current(): number {
  return 1;
}
const currentValue = current();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-deprecated": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_deprecated.rs#L5)
