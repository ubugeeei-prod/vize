---
title: "script/custom-event-name-casing"
description: "Imposer camelCase aux noms des événements personnalisés émis."
---

# `script/custom-event-name-casing`

Imposer camelCase aux noms des événements personnalisés émis.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

La chaîne émise `my-event` contient un tiret et enfreint la convention camelCase par défaut pour les noms d’événements.

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits(['my-event'])
emit('my-event')         // kebab-case → report
</script>
```

## Bon

La déclaration et l’appel utilisent tous deux `myEvent`, ce qui maintient la correspondance entre le nom de l’événement et son émission tout en respectant la convention de casse par défaut. Une convention kebab-case configurée impose une autre forme.

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits(['myEvent'])
emit('myEvent')
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
        "script/custom-event-name-casing": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L61)
