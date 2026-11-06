---
title: "html/no-consecutive-br"
description: "Interdire les éléments &lt;br&gt; consécutifs."
---

# `html/no-consecutive-br`

Interdire les éléments &lt;br&gt; consécutifs.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Deux éléments de saut de ligne consécutifs créent un espacement entre des blocs au sein d’un seul paragraphe.

```vue annotate="remove:2"
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

## Bon

Des paragraphes séparés expriment les deux blocs de contenu sans répéter les éléments de saut de ligne.

```vue annotate="add:2,3"
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/no-consecutive-br": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles HTML](../html.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30)
