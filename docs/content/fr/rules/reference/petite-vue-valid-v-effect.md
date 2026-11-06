---
title: "petite-vue/valid-v-effect"
description: "Exiger une expression non vide pour v-effect."
---

# `petite-vue/valid-v-effect`

Exiger une expression non vide pour v-effect.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | Aucune |

Champ d’application : Documents HTML détectés comme petite-vue ; les SFC Vue ordinaires sont hors du champ de cette règle.

## Mauvais

Chaque `v-effect` ne possède aucune expression exécutable : sa valeur est absente, vide ou composée uniquement d’espaces.

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-effect></div>
<div v-effect=""></div>
<div v-effect="   "></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## Bon

Les deux valeurs de `v-effect` contiennent une expression : l’une met à jour `el.textContent` et l’autre incrémente `count`. Cette règle vérifie que l’expression n’est pas vide, et non la logique métier de l’effet.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "petite-vue/valid-v-effect": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles petite-vue](../petite-vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L37)
