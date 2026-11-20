---
title: "musea/require-title"
description: "Exiger un titre d’affichage résoluble pour le bloc &lt;art&gt;."
---

# `musea/require-title`

Exiger un titre d’affichage résoluble pour le bloc &lt;art&gt;.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | Aucune |

Champ d’application : Blocs art, variant et style des fichiers .art.vue de Musea.

## Mauvais

Le bloc art ne fournit ni métadonnées explicites ni composant permettant d’inférer un titre.

```vue annotate="remove:2"
<art>
  <variant name="primary"><div /></variant>
</art>
```

## Bon

La cible Button importée fournit le titre d’affichage inféré.

```vue annotate="add:1,2,3,4,6"
<script setup>
import Button from "./Button.vue";
</script>

<art>
  <variant name="primary"><Button /></variant>
</art>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "musea/require-title": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Musea](../musea.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L32)
