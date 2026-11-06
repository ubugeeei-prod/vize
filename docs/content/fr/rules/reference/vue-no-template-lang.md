---
title: "vue/no-template-lang"
description: "Déconseiller l’attribut lang sur le bloc template."
---

# `vue/no-template-lang`

Déconseiller l’attribut lang sur le bloc template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le template sélectionne Pug avec lang. Il s’agit d’une convention souhaitée limitant les templates au HTML ; le traitement SFC actuel n’émet pas de diagnostic pour cet identifiant du catalogue.

```vue annotate="remove:1,2"
<template lang="pug">
p Notice
</template>
```

## Bon

Un template HTML ordinaire omet lang et utilise directement le paragraphe. Cela illustre la convention sans prétendre qu’un diagnostic SFC est actuellement émis.

```vue annotate="add:1,2"
<template>
<p>Notice</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-template-lang": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L11)
