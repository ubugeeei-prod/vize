---
title: "a11y/label-has-for"
description: "Exiger des contrôles de formulaire associés aux libellés."
---

# `a11y/label-has-for`

Exiger des contrôles de formulaire associés aux libellés.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Listez les composants qui rendent un contrôle de formulaire natif et satisfont donc l'imbrication du label. Les écritures PascalCase et kebab-case sont équivalentes. Les options n'activent pas la règle ; les couches suivantes remplacent la liste entière et [] efface les noms hérités.

## Mauvais

Le label séparé n’est ni associé au moyen de `for` ni placé autour du champ.

```vue annotate="remove:2"
<template>
<label>Name</label>
</template>
```

## Bon

Le champ input natif imbriqué est implicitement associé au label qui l’entoure.

```vue annotate="add:2"
<template>
<label>Name <input v-model="name" /></label>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/label-has-for": "warn"
    },
    "ruleOptions": {
      "a11y/label-has-for": {
        "controlComponents": [
          "CInputField"
        ]
      }
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L32)
