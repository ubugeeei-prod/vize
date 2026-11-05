---
title: "musea/valid-variant"
description: "Exiger un attribut name dans les blocs &lt;variant&gt;."
---

# `musea/valid-variant`

Exiger un attribut name dans les blocs &lt;variant&gt;.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | Aucune |

Champ d’application : Blocs art, variant et style des fichiers .art.vue de Musea.

## Mauvais

Le variant omet le nom nécessaire pour identifier l’aperçu.

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

## Bon

Le nom primary identifie ce variant.

```vue annotate="add:2"
<art title="Button" component="./Button.vue">
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
        "musea/valid-variant": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Musea et CSS](../musea-and-css.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8)
