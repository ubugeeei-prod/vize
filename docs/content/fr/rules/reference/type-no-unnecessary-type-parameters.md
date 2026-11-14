---
title: "type/no-unnecessary-type-parameters"
description: "Interdire les paramètres de type qui ne sont pas utilisés plus d’une fois."
---

# `type/no-unnecessary-type-parameters`

Interdire les paramètres de type qui ne sont pas utilisés plus d’une fois.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

T n’apparaît qu’une fois dans la signature et n’exprime donc aucune relation entre les types.

```vue annotate="remove:2,3"
<script setup lang="ts">
function logValue<T>(value: T): void {
  console.info(value);
}
</script>
```

## Bon

T relie le type du paramètre au type de retour.

```vue annotate="add:2,3"
<script setup lang="ts">
function identity<T>(value: T): T {
  return value;
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
      "type/no-unnecessary-type-parameters": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_parameters.rs#L5)
