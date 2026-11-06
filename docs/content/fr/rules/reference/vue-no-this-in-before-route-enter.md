---
title: "vue/no-this-in-before-route-enter"
description: "Interdit d'accéder par `this` à une instance encore inexistante dans `beforeRouteEnter`."
---

# `vue/no-this-in-before-route-enter`

Interdit d'accéder par `this` à une instance encore inexistante dans `beforeRouteEnter`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`beforeRouteEnter` s'exécute avant que l'instance du composant soit disponible ; `this.load()` ne peut donc pas y accéder.

```vue annotate="remove:4,5"
<script lang="ts">
export default {
  beforeRouteEnter(to, from, next) {
    this.load();
    next();
  },
  methods: { load() {} },
};
</script>
```

## Bon

Utilisez l'instance passée au callback de `next` et appelez `vm.load()` après la création du composant.

```vue annotate="add:4"
<script lang="ts">
export default {
  beforeRouteEnter(to, from, next) {
    next((vm) => vm.load());
  },
  methods: { load() {} },
};
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-this-in-before-route-enter": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_this_in_before_route_enter.rs#L24)
