---
title: "type/no-meaningless-void-operator"
description: "Interdire l’opérateur void sur une expression qui est déjà void."
---

# `type/no-meaningless-void-operator`

Interdire l’opérateur void sur une expression qui est déjà void.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

logVoid() est déjà void, donc l’opérateur void ne change pas le résultat.

```vue annotate="remove:2,3"
<script setup lang="ts">
function logVoid(): void {}
void logVoid();
</script>
```

## Bon

getNumber() renvoie une Promise, donc void abandonne ce thenable volontairement.

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
void getNumber();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-meaningless-void-operator": "warn"
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_meaningless_void_operator.rs#L5)
