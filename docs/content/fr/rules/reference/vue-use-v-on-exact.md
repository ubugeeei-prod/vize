---
title: "vue/use-v-on-exact"
description: "Imposer le modificateur `.exact` sur `v-on` en présence de gestionnaires utilisant des modificateurs."
---

# `vue/use-v-on-exact`

Imposer le modificateur `.exact` sur `v-on` en présence de gestionnaires utilisant des modificateurs.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Quand les gestionnaires se chevauchent

La règle compare les écouteurs du même événement sur un élément natif. Elle ignore les événements des composants personnalisés, sauf si les deux écouteurs utilisent `.native`. Des noms d’événements différents et des filtres de touches sans touche commune n’entrent pas en conflit.

Les modificateurs système désignent des touches qui doivent être enfoncées ; `@keydown.alt.shift` accepte donc également Ctrl+Alt+Shift. Si un second gestionnaire utilise `@keydown.alt.ctrl.shift`, ajoutez `.exact` au gestionnaire le plus général pour séparer ces actions. `.exact` autorise les modificateurs système qu’il déclare et refuse toute touche Ctrl, Shift, Alt ou Meta supplémentaire.

Plusieurs modificateurs de touches représentent des alternatives : `@keydown.up.down` accepte l’une ou l’autre des deux touches fléchées. Il peut se chevaucher avec `@keydown.ctrl.down` sur Ctrl+FlècheBas. La règle signale la clé de directive qui nécessite `.exact`, sans inclure son expression de gestionnaire.

## Mauvais

Le gestionnaire de clic ordinaire peut également s’exécuter lors d’un Ctrl-clic et chevaucher le gestionnaire `.ctrl` distinct.

```vue annotate="remove:2"
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

## Bon

`.exact` limite le gestionnaire de clic ordinaire aux clics sans touche modificatrice ; le gestionnaire propre à Ctrl reste distinct.

```vue annotate="add:2,3,4,5,6"
<template>
  <button
    type="button"
    @click.exact="handleClick"
    @click.ctrl="handleCtrlClick"
  >
    Save
  </button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/use-v-on-exact": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L31)
