---
title: "vue/warn-custom-directive"
description: "Signaler les directives personnalisées nécessitant un enregistrement."
---

# `vue/warn-custom-directive`

Signaler les directives personnalisées nécessitant un enregistrement.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Listez les noms exacts des directives enregistrées globalement, sans le préfixe v-. Les bindings vName locaux de script-setup restent détectés automatiquement. Les options n'activent pas la règle ; les couches suivantes remplacent la liste entière et [] efface les noms hérités.

## Mauvais

`v-focus`, `v-mask` et `v-click-outside` nécessitent des implémentations de directives propres au projet, signalées par cette convention facultative.

```vue annotate="remove:2"
<template>
<button type="button" v-focus>Focus</button>
</template>
```

## Bon

L’exemple utilise les directives intégrées `v-if`, `v-model` et `v-on`. Une directive personnalisée correctement enregistrée peut rester valide en Vue lorsque cette politique est désactivée.

```vue annotate="add:2"
<template>
<button type="button" v-hover-tooltip="tip">Copy</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/warn-custom-directive": "warn"
    },
    "ruleOptions": {
      "vue/warn-custom-directive": {
        "ignore": [
          "hover-tooltip"
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44)
