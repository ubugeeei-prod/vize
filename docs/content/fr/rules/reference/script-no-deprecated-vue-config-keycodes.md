---
title: "script/no-deprecated-vue-config-keycodes"
description: "Interdire Vue.config.keyCodes, retiré dans Vue 3."
---

# `script/no-deprecated-vue-config-keycodes`

Interdire Vue.config.keyCodes, retiré dans Vue 3.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`Vue.config.keyCodes.enter` affecte un alias de code de touche retiré.

```vue annotate="remove:2"
<script setup lang="ts">
Vue.config.keyCodes.enter = 13
</script>
```

## Bon

Le code de touche est une constante ordinaire, `const enter = 13`.

```vue annotate="add:2"
<script setup lang="ts">
const enter = 13
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-vue-config-keycodes": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_vue_config_keycodes.rs#L22)
