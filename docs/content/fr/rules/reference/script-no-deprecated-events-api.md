---
title: "script/no-deprecated-events-api"
description: "Interdire l’API d’événements de Vue 2 supprimée ($on / $off / $once)."
---

# `script/no-deprecated-events-api`

Interdire l’API d’événements de Vue 2 supprimée ($on / $off / $once).

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Les appels `$on`, `$once` et `$off` utilisent les méthodes de bus d’événements de l’instance supprimées dans Vue 3.

```vue annotate="remove:4,5,6,7"
<script>
export default {
  mounted() {
    this.$on('event', handler)
    const vm = this
    vm.$once('event', handler)
    this.$off('event', handler)
  }
}
</script>
```

## Bon

`$emit` reste valide, tandis que l’abonnement au bus d’événements passe à la méthode `on` de l’émetteur externe. La correction sépare l’émission destinée au parent du bus d’événements externe.

```vue annotate="add:2,3,6,7"
<script>
import mitt from 'mitt'
const emitter = mitt()
export default {
  mounted() {
    this.$emit('event', payload)
    emitter.on('event', handler)
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
      "script/no-deprecated-events-api": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L51)
