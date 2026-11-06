---
title: "script/no-potential-component-option-typo"
description: "Signaler les fautes de frappe probables dans les noms d’options de composant de l’Options API."
---

# `script/no-potential-component-option-typo`

Signaler les fautes de frappe probables dans les noms d’options de composant de l’Options API.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

L’option est écrite `method`, à une modification près de l’option reconnue `methods` ; Vue ne la traiterait pas comme la déclaration de méthodes souhaitée.

```vue annotate="remove:2"
<script lang="ts">
export default { method: { save() {} } };
</script>
```

## Bon

Changer la clé en `methods` place `save()` sous l’option de composant reconnue.

```vue annotate="add:2"
<script lang="ts">
export default { methods: { save() {} } };
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-potential-component-option-typo": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19)
