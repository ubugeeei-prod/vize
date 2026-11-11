---
title: "type/no-unnecessary-type-arguments"
description: "Interdire un argument de type égal à la valeur par défaut."
---

# `type/no-unnecessary-type-arguments`

Interdire un argument de type égal à la valeur par défaut.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

L’argument explicite number est identique au paramètre de type par défaut.

```vue annotate="remove:5"
<script setup lang="ts">
function identity<T = number>(value: T): T {
  return value;
}
const fromIdentity = identity<number>(1);
</script>
```

## Bon

Appeler identity(1) laisse s’appliquer le paramètre de type par défaut.

```vue annotate="add:5"
<script setup lang="ts">
function identity<T = number>(value: T): T {
  return value;
}
const fromIdentity = identity(1);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-arguments": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_arguments.rs#L5)
