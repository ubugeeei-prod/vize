---
title: "vue/no-script-non-standard-lang"
description: "Déconseiller les valeurs lang non standard dans les scripts."
---

# `vue/no-script-non-standard-lang`

Déconseiller les valeurs lang non standard dans les scripts.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non implémentée pour le lint des SFC | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

> [!WARNING]
> Prise en charge actuelle : `no-sfc-finding`. Vize accepte cet identifiant de règle dans la configuration, mais le lint des SFC ne le signale pas encore. Le bon exemple montre la convention visée.

Cette entrée du catalogue n’émet actuellement aucun diagnostic propre à cette règle lors du lint des SFC. La paire Mauvais/Bon décrit la convention souhaitée, et non un diagnostic exécutable. Activer l’identifiant ne fournit pas le contrôle SFC manquant.

## Mauvais

Le script utilise la syntaxe CoffeeScript avec lang=coffee. Le traitement SFC actuel n’émet pas cette règle du catalogue pour ce langage.

```vue annotate="remove:1,2"
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

## Bon

Le script utilise une déclaration TypeScript ordinaire avec lang=ts, illustrant la convention de langage souhaitée.

```vue annotate="add:1,2"
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

## ID configuré (aucun diagnostic SFC actuellement)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-script-non-standard-lang": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44)
