---
title: "type/no-floating-promises"
description: "Interdire les Promises laissées sans traitement."
---

# `type/no-floating-promises`

Interdire les Promises laissées sans traitement.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

La fonction asynchrone `save` renvoie une Promise, mais l’appel isolé `save()` ne l’attend ni ne la renvoie, et ne marque pas explicitement son abandon volontaire.

```vue annotate="remove:3"
<script setup lang="ts">
async function save(): Promise<void> {}
save();
</script>
```

## Bon

`void save()` marque explicitement l’intention de lancer l’opération sans attendre son résultat, acceptée par cette règle. Il s’agit d’un marqueur explicite d’abandon, et non d’un gestionnaire de rejet.

```vue annotate="add:3"
<script setup lang="ts">
async function save(): Promise<void> {}
void save();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-floating-promises": "warn"
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13)
