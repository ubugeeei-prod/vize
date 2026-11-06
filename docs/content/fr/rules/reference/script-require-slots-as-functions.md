---
title: "script/require-slots-as-functions"
description: "Exiger que les propriétés de $slots soient appelées comme des fonctions."
---

# `script/require-slots-as-functions`

Exiger que les propriétés de $slots soient appelées comme des fonctions.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`render` renvoie `this.$slots.default` sans l’appeler, ce qui était le tableau de vnodes de Vue 2.

```vue annotate="remove:4"
<script lang="ts">
export default {
  render() {
    return this.$slots.default
  }
}
</script>
```

## Bon

`render` appelle `this.$slots.default()` et utilise le vnode renvoyé.

```vue annotate="add:4"
<script lang="ts">
export default {
  render() {
    return this.$slots.default()
  }
}
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-slots-as-functions": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_slots_as_functions.rs#L44)
