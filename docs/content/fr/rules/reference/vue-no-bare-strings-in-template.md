---
title: "vue/no-bare-strings-in-template"
description: "Interdire le texte brut destiné aux utilisateurs dans les templates lorsqu’il devrait être internationalisé."
---

# `vue/no-bare-strings-in-template`

Interdire le texte brut destiné aux utilisateurs dans les templates lorsqu’il devrait être internationalisé.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Lorsqu'elle est présente, allowlist remplace les valeurs de ponctuation par défaut d'eslint-plugin-vue. Les entrées sont des sous-chaînes littérales ou des valeurs /pattern/flags retirées avant la vérification du texte restant. Omettez allowlist pour conserver les valeurs par défaut ; [] les efface. Les options n'activent pas la règle.

## Mauvais

Le texte visible contient une chaîne non traduite directement dans le template.

```vue annotate="remove:2"
<template>
<p>Translate me</p>
</template>
```

## Bon

Le libellé de la touche provient de `$t` au lieu d’une chaîne brute dans le template.

```vue annotate="add:2"
<template>
<kbd>{{ $t("keyboard.forward") }}</kbd>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-bare-strings-in-template": "warn"
    },
    "ruleOptions": {
      "vue/no-bare-strings-in-template": {
        "allowlist": [
          "/^[DRWXYmvx]$/u"
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L52)
