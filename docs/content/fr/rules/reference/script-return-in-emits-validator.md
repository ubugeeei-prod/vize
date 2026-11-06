---
title: "script/return-in-emits-validator"
description: "Exiger une valeur de retour dans chaque validateur emits de l’Options API."
---

# `script/return-in-emits-validator`

Exiger une valeur de retour dans chaque validateur emits de l’Options API.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

Utilisez une fonction fléchée à corps de bloc pour le filtre SFC actuellement pris en charge. Le validateur sous-jacent gère aussi la syntaxe abrégée des méthodes, mais le préfiltre SFC actuel ne transmet pas cette forme de façon fiable.

## Mauvais

Le validateur `submit` journalise la charge utile mais ne renvoie aucun résultat de validation ; son corps de bloc produit donc undefined.

```vue annotate="remove:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { console.log(payload); } } };
</script>
```

## Bon

`return payload != null` fournit un résultat de validation booléen pour la charge utile soumise au lieu de se terminer sans valeur de retour.

```vue annotate="add:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { return payload != null; } } };
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/return-in-emits-validator": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L61)
