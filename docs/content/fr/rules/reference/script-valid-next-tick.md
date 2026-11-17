---
title: "script/valid-next-tick"
description: "Exiger que le résultat d’un appel nextTick() soit attendu, chaîné ou associé à un callback."
---

# `script/valid-next-tick`

Exiger que le résultat d’un appel nextTick() soit attendu, chaîné ou associé à un callback.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

L’appel à `nextTick()` importé est une expression seule sans callback ; la Promise renvoyée est donc ignorée et aucun traitement n’attend l’application des mises à jour du DOM.

```vue annotate="remove:3"
<script setup lang="ts">
import { nextTick } from "vue";
nextTick();
</script>
```

## Bon

`await nextTick()` consomme la Promise et attend explicitement la prochaine mise à jour du DOM avant que le code setup suivant ne continue.

```vue annotate="add:3"
<script setup lang="ts">
import { nextTick } from "vue";
await nextTick();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/valid-next-tick": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55)
