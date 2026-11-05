---
title: "vapor/require-vapor-attribute"
description: "Suggérer l’ajout de l’attribut vapor à script setup."
---

# `vapor/require-vapor-attribute`

Suggérer l’ajout de l’attribut vapor à script setup.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non implémentée pour le lint des SFC | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

> [!WARNING]
> Prise en charge actuelle : `no-sfc-finding`. Vize accepte cet identifiant de règle dans la configuration, mais le lint des SFC ne le signale pas encore. Le bon exemple montre la convention visée.

Cette règle est un emplacement réservé doté d’un callback vide. Ajouter vapor sélectionne la compilation Vapor ; le linter actuel ne signale pas cet identifiant du catalogue lorsque vapor est absent.

## Mauvais

Le bloc script setup ne possède pas l’attribut de compilation Vapor. Il s’agit d’une convention prévue : la fonction de rappel actuellement vide de la règle ne le signale pas.

```vue annotate="remove:1"
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

## Bon

L’ajout de vapor sélectionne la compilation Vapor. Il illustre la correction prévue et ne signifie pas que le linter actuel émet cette règle du catalogue.

```vue annotate="add:1"
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

## ID configuré (aucun diagnostic SFC actuellement)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vapor/require-vapor-attribute": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vapor](../vapor.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17)
