---
title: "type/await-thenable"
description: "Interdire d’await une valeur qui n’est pas thenable."
---

# `type/await-thenable`

Interdire d’await une valeur qui n’est pas thenable.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

plainNumber renvoie un nombre, donc await attend une valeur qui n’est pas thenable.

```vue annotate="remove:2,3,4,6"
<script setup lang="ts">
function plainNumber(): number {
  return 1;
}
async function run(): Promise<number> {
  return await plainNumber();
}
</script>
```

## Bon

Promise.resolve(1) est thenable, donc await attend une vraie Promise.

```vue annotate="add:3"
<script setup lang="ts">
async function run(): Promise<number> {
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
      "type/await-thenable": "warn"
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/await_thenable.rs#L5)
