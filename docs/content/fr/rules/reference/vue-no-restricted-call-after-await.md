---
title: "vue/no-restricted-call-after-await"
description: "Interdit d'appeler une méthode restreinte après await."
---

# `vue/no-restricted-call-after-await`

Interdit d'appeler une méthode restreinte après await.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

setup appelle onMounted après await.

```vue annotate="remove:4,5"
<script lang="ts">
import { defineComponent, onMounted } from "vue";
export default defineComponent({
  async setup() {
    await Promise.resolve();
    onMounted(() => {});
  },
});
</script>
<template>
  <p>ok</p>
</template>
```

## Bon

setup enregistre onMounted sans await.

```vue annotate="add:4"
<script lang="ts">
import { defineComponent, onMounted } from "vue";
export default defineComponent({
  setup() {
    onMounted(() => {});
  },
});
</script>
<template>
  <p>ok</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-call-after-await": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-call-after-await": [
        {
          "module": "vue",
          "path": "onMounted"
        }
      ]
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/await_call.rs#L25)
