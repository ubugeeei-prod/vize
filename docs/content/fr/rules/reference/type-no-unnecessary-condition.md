---
title: "type/no-unnecessary-condition"
description: "Interdire une condition toujours vraie, toujours fausse, never ou toujours nullish."
---

# `type/no-unnecessary-condition`

Interdire une condition toujours vraie, toujours fausse, never ou toujours nullish.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

value est une chaîne, donc elle n’est jamais nullish et le repli de ?? ne s’exécute jamais.

```vue annotate="remove:2"
<script setup lang="ts">
function orDefault(value: string): string {
  return value ?? "x";
}
</script>
```

## Bon

value peut être undefined, donc ?? fournit une chaîne lorsqu’elle est absente.

```vue annotate="add:2"
<script setup lang="ts">
function orDefault(value: string | undefined): string {
  return value ?? "x";
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
      "type/no-unnecessary-condition": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_condition.rs#L5)
