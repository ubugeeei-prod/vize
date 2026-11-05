---
title: "musea/require-title"
description: "Exiger l’attribut title dans le bloc &lt;art&gt;."
---

# `musea/require-title`

Exiger l’attribut title dans le bloc &lt;art&gt;.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | Aucune |

Champ d’application : Blocs art, variant et style des fichiers .art.vue de Musea.

## Mauvais

Le bloc art identifie Button.vue, mais ne fournit aucun titre.

```vue annotate="remove:1"
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

## Bon

Les options de defineArt fournissent le titre Button pour le bloc art.

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "musea/require-title": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Musea et CSS](../musea-and-css.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31)
