---
title: "vue/max-template-depth"
description: "Limite la profondeur du template."
---

# `vue/max-template-depth`

Limite la profondeur du template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le span est imbriqué au-delà du maximum configuré.

```vue annotate="remove:2,3,4,5,6,7,8"
<template>
  <div>
    <section>
      <div>
        <span>deep</span>
      </div>
    </section>
  </div>
</template>
```

## Bon

Le template n'a qu'un élément.

```vue annotate="add:2"
<template>
  <p>ok</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/max-template-depth": "warn"
    },
    "ruleOptions": {
      "vue/max-template-depth": {
        "maxDepth": 3
      }
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/max_template_depth.rs#L13)
