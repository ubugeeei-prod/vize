---
title: "type/no-unnecessary-type-parameters"
description: "Interdire un paramètre de type utilisé une seule fois dans sa signature."
---

# `type/no-unnecessary-type-parameters`

Interdire un paramètre de type utilisé une seule fois dans sa signature.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

T n’apparaît qu’une fois dans la signature de logValue, donc il ne relie rien.

```vue annotate="remove:2"
<script setup lang="ts">
function logValue<T>(value: T): void {
  console.info(value);
}
logValue(1);
</script>
```

## Bon

Typer value en unknown accepte les mêmes arguments sans paramètre de type.

```vue annotate="add:2"
<script setup lang="ts">
function logValue(value: unknown): void {
  console.info(value);
}
logValue(1);
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
