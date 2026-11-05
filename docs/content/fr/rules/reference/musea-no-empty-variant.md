---
title: "musea/no-empty-variant"
description: "Interdire les blocs &lt;variant&gt; vides."
---

# `musea/no-empty-variant`

Interdire les blocs &lt;variant&gt; vides.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Blocs art, variant et style des fichiers .art.vue de Musea.

## Mauvais

Le variant nommé primary est vide et ne fournit donc aucun contenu d’aperçu.

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

## Bon

Le variant affiche un Button primary avec son contenu Save.

```vue annotate="add:2,3,4"
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
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
        "musea/no-empty-variant": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Musea et CSS](../musea-and-css.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8)
