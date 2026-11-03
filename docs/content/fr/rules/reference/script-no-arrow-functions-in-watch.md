---
title: "script/no-arrow-functions-in-watch"
description: "Interdire les fonctions fléchées comme gestionnaires watch de l’Options API."
---

# `script/no-arrow-functions-in-watch`

Interdire les fonctions fléchées comme gestionnaires watch de l’Options API.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Le watcher `value` de l’Options API et le gestionnaire imbriqué `other.handler` sont des fonctions fléchées. Une fonction fléchée capture le `this` du contexte environnant au lieu de recevoir l’instance du composant.

```vue annotate="remove:4,5,9"
<script lang="ts">
export default {
  watch: {
    // `this` is not the component instance inside an arrow function.
    value: () => {
      this.doSomething()
    },
    other: {
      handler: () => {}
    }
  }
}
</script>
```

## Bon

Les deux gestionnaires deviennent des méthodes ordinaires, ce qui permet à Vue de lier `this` au composant. L’option du watcher `deep: true` reste compatible avec la forme objet.

```vue annotate="add:4,8,9"
<script lang="ts">
export default {
  watch: {
    value(newValue, oldValue) {
      this.doSomething()
    },
    other: {
      handler(newValue) {},
      deep: true
    }
  }
}
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
        "script/no-arrow-functions-in-watch": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_arrow_functions_in_watch.rs#L59)
