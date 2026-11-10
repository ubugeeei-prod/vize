---
title: "vue/no-deprecated-v-on-number-modifiers"
description: "Interdire les modificateurs numériques obsolètes `keyCode` sur `v-on`."
---

# `vue/no-deprecated-v-on-number-modifiers`

Interdire les modificateurs numériques obsolètes `keyCode` sur `v-on`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Oui | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Sélection des modificateurs numériques et corrections

Le premier modificateur numérique dont le nom commence par un entier décimal détermine le diagnostic. `@keyup.13.27="submit"` ne signale que `13` et propose `@keyup.enter.27="submit"` ; un autre passage de correction peut remplacer `27` par `escape`. `@keyup.13suffix="submit"` devient également `@keyup.enter="submit"`. Les valeurs de 0 à 9 sont exemptées ; `@keyup.0.13="submit"` ne produit donc aucun diagnostic de cette règle. Les valeurs inconnues telles que `999` sont signalées sans correction. L’exemption existante pour Vue 2 et petite-vue reste en vigueur.

## Mauvais

Les gestionnaires de clavier identifient les touches par les codes numériques supprimés 13 et 27.

```vue annotate="remove:2,3,4"
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

## Bon

Les gestionnaires utilisent les modificateurs de touches nommés enter et esc.

```vue annotate="add:2,3"
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-v-on-number-modifiers": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L46)
