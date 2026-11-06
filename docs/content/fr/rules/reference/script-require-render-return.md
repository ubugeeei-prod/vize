---
title: "script/require-render-return"
description: "Exiger que la fonction render du composant renvoie une valeur."
---

# `script/require-render-return`

Exiger que la fonction render du composant renvoie une valeur.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

La fonction `render` lit `this.count` dans une branche et ne renvoie rien.

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
  render() {
    if (this.ready) {
      this.count
    }
  }
}
</script>
```

## Bon

La fonction `render` renvoie `this.count`.

```vue annotate="add:4"
<script lang="ts">
export default {
  render() {
    return this.count
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
      "script/require-render-return": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_render_return.rs#L35)
