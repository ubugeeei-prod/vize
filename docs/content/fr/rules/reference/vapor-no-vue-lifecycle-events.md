---
title: "vapor/no-vue-lifecycle-events"
description: "Interdire les événements de cycle de vie @vue:xxx par élément (non pris en charge dans Vapor)."
---

# `vapor/no-vue-lifecycle-events`

Interdire les événements de cycle de vie @vue:xxx par élément (non pris en charge dans Vapor).

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le champ de saisie utilise l’événement de cycle de vie de template @vue:mounted.

```vue annotate="remove:2"
<template>
  <input @vue:mounted="focusInput" />
</template>
```

## Bon

onMounted accède à la référence de template nommée et donne le focus au champ de saisie au moyen du hook de cycle de vie de script pris en charge.

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vapor/no-vue-lifecycle-events": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vapor](../vapor.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34)
