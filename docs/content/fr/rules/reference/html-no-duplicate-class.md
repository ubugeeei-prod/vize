---
title: "html/no-duplicate-class"
description: "Interdire les noms de classes en double dans un attribut class statique."
---

# `html/no-duplicate-class`

Interdire les noms de classes en double dans un attribut class statique.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La liste de classes statique répète le nom `btn`.

```vue annotate="remove:2"
<template>
  <div class="btn btn primary">click</div>
</template>
```

## Bon

La liste de classes conserve une seule occurrence de `btn` et le nom distinct `primary`.

```vue annotate="add:2"
<template>
  <div class="btn primary">click</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/no-duplicate-class": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles HTML](../html.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L34)
