---
title: "vue/no-bare-strings-in-template"
description: "Interdire le texte brut destiné aux utilisateurs dans les templates lorsqu’il devrait être internationalisé."
---

# `vue/no-bare-strings-in-template`

Interdire le texte brut destiné aux utilisateurs dans les templates lorsqu’il devrait être internationalisé.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le texte visible et les attributs de nommage contiennent des chaînes non traduites directement dans le template.

```vue annotate="remove:2,3,4,5"
<template>
<div>hello</div>
<img alt="a cat" />
<input placeholder="Search" />
<button title="Close">x</button>
</template>
```

## Bon

Le contenu traduisible appelle $t ; les exemples de ponctuation et de valeurs uniquement numériques sont des exceptions autorisées.

```vue annotate="add:2,3,4,5,6"
<template>
<div>{{ $t('hello') }}</div>
<img :alt="$t('cat')" />
<div>-</div>
<div>123</div>
<button :title="$t('close')">{{ $t('x') }}</button>
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
        "vue/no-bare-strings-in-template": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L47)
