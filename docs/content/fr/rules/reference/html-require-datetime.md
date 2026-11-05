---
title: "html/require-datetime"
description: "Exiger l’attribut datetime sur l’élément &lt;time&gt;."
---

# `html/require-datetime`

Exiger l’attribut datetime sur l’élément &lt;time&gt;.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L’élément time contient une date lisible par une personne, mais aucune valeur datetime lisible par une machine.

```vue annotate="remove:2"
<template>
  <time>May 13, 2026</time>
</template>
```

## Bon

`datetime="2026-05-13"` fournit la date correspondante lisible par une machine.

```vue annotate="add:2"
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "html/require-datetime": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles HTML](../html.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L36)
