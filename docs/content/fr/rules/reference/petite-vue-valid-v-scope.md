---
title: "petite-vue/valid-v-scope"
description: "Exiger que v-scope reçoive un objet littéral."
---

# `petite-vue/valid-v-scope`

Exiger que v-scope reçoive un objet littéral.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | Aucune |

Champ d’application : Documents HTML détectés comme petite-vue ; les SFC Vue ordinaires sont hors du champ de cette règle.

## Mauvais

Les quatre valeurs non vides de `v-scope` sont un identifiant, un appel, une expression arithmétique et un nombre ; aucune n’est analysée comme un objet littéral.

```html annotate="remove:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope="count"></div>
<div v-scope="foo()"></div>
<div v-scope="a + b"></div>
<div v-scope="123"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## Bon

Un `v-scope` sans valeur utilise la portée racine. Les autres valeurs sont des objets littéraux, y compris l’objet entre parenthèses, que la règle accepte.

```html annotate="add:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope></div>
<div v-scope="{}"></div>
<div v-scope="{ count: 0 }"></div>
<div v-scope="({ count: 0 })"></div>
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
      "petite-vue/valid-v-scope": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles petite-vue](../petite-vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46)
