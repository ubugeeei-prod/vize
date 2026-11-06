---
title: "vue/valid-v-on"
description: "Exiger des directives `v-on` valides."
---

# `vue/valid-v-on`

Exiger des directives `v-on` valides.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Validation des modificateurs

Les modificateurs non pris en charge sont signalés individuellement sur leur nom, y compris avec la forme longue `v-on:click`, les arguments d’événement dynamiques et la syntaxe objet des écouteurs. Les noms acceptés correspondent à ceux d’`eslint-plugin-vue` 10.9.2 : les modificateurs d’événement intégrés de Vue, la table complète des alias de touches, tout point de code Unicode unique et les noms ayant un préfixe entier décimal fini (selon la sémantique de `Number.parseInt(name, 10)` en JavaScript). Par exemple, cette règle accepte `.13suffix`. Les diagnostics concernant l’abandon des codes de touche numériques et la migration de `.native` relèvent de règles distinctes.

Seuls `.stop` et `.prevent` ont un effet sans gestionnaire. Un modificateur tel que `.once`, `.enter` ou un nom personnalisé configuré nécessite toujours une expression de gestionnaire.

Les projets ayant une gestion personnalisée des événements peuvent ajouter des noms sous `vue.linter.ruleOptions` :

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/valid-v-on": "error" },
    ruleOptions: { "vue/valid-v-on": { modifiers: ["custom"] } },
  },
});
```

Cela autorise `@click.custom="handler"`. Les noms doivent correspondre exactement, en respectant la casse ; les options complètent les noms intégrés et n’activent pas une règle désactivée. Les entrées de configuration à portée limitée peuvent remplacer les noms hérités par une nouvelle liste ou les réinitialiser avec `[]`.

La table des touches est adaptée des [alias de touches d’eslint-plugin-vue](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/key-aliases.json), sous licence MIT. Son code source et sa licence sont conservés aux côtés de l’implémentation.

## Mauvais

Les formes d’écouteurs omettent un argument d’événement ou l’expression de gestionnaire ou d’objet nécessaire.

```vue annotate="remove:2,3,4,5,6,7"
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
  <div @click.sync="handleClick"></div>
  <div @click.nativeish="handleClick"></div>
  <div @click.once></div>
</template>
```

## Bon

Utilisez un événement avec son gestionnaire, ou transmettez un objet d’écouteurs à `v-on` sans argument.

```vue annotate="add:2,3,4,5,6"
<template>
  <div @click="handleClick"></div>
  <form @submit.prevent></form>
  <input @keyup.page-down="handlePageDown" />
  <input @keyup.あ="handleKey" />
  <div v-on="{ click: handleClick }"></div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-on": "error"
    },
    "ruleOptions": {
      "vue/valid-v-on": {
        "modifiers": [
          "custom"
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30)
