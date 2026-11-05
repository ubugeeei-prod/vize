---
title: "script/define-props-declaration"
description: "Imposer la forme typée defineProps&lt;{ ... }&gt;() à la place de la forme à l’exécution sous forme d’objet."
---

# `script/define-props-declaration`

Imposer la forme typée defineProps&lt;{ ... }&gt;() à la place de la forme à l’exécution sous forme d’objet.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`defineProps({ title: String })` fournit un objet à l’exécution, ce qui va à l’encontre de la préférence de cette règle pour les props typées.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ title: String });
console.log(props.title);
</script>
```

## Bon

`defineProps<{ title: string }>()` déclare `title` dans l’argument de type et conserve l’accès `props.title` sans argument de déclaration à l’exécution.

```vue annotate="add:2"
<script setup lang="ts">
const props = defineProps<{ title: string }>();
console.log(props.title);
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
        "script/define-props-declaration": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40)
