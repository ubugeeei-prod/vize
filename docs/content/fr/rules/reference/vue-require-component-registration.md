---
title: "vue/require-component-registration"
description: "Exiger un import ou un enregistrement explicite des composants."
---

# `vue/require-component-registration`

Exiger un import ou un enregistrement explicite des composants.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Listez les noms explicites des composants fournis par les plugins de l’application ou par previewSetup de Musea. Les graphies PascalCase et kebab-case sont acceptées ; globals contient des noms, pas des expressions régulières. ignorePatterns prend des expressions régulières, comme vue/no-undef-components d’eslint-plugin-vue. Un attribut `is` statique nomme un composant sur `<component>` ou avec le préfixe `vue:` ; sur un autre élément, c’est un élément natif personnalisé et il n’est pas vérifié. Les options n’activent pas la règle. Les couches suivantes remplacent la liste ; une liste vide efface les noms hérités.

## Mauvais

`MissingWidget` n’est ni enregistré ni inclus dans la liste configurée des composants globaux autorisés.

```vue annotate="remove:2"
<template>
<MissingWidget />
</template>
```

## Bon

`MyButton` figure dans l’option `globals` de l’exemple. Cette option exempte un composant global connu ; elle ne l’enregistre ni ne l’importe.

```vue annotate="add:2"
<template>
<MyButton />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-component-registration": "warn"
    },
    "ruleOptions": {
      "vue/require-component-registration": {
        "globals": [
          "MyButton",
          "MyIcon"
        ]
      }
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L59)
