---
title: "vue/no-multiple-template-root"
description: "Interdire plusieurs nœuds racines dans un template."
---

# `vue/no-multiple-template-root`

Interdire plusieurs nœuds racines dans un template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Activez cette règle uniquement pour un contrat à racine unique. Vue 3 prend normalement en charge les fragments.

## Mauvais

La convention à racine unique, activée explicitement, trouve deux paragraphes frères à la racine du template.

```vue annotate="remove:2,3"
<template>
<p>First</p>
<p>Second</p>
</template>
```

## Bon

Un élément section enveloppe les paragraphes dans une seule racine ; activez cette convention uniquement lorsqu’un contrat à racine unique est souhaité.

```vue annotate="add:2"
<template>
<section><p>First</p><p>Second</p></section>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-multiple-template-root": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L22)
