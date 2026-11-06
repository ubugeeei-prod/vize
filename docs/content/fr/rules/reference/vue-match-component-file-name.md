---
title: "vue/match-component-file-name"
description: "Exige que le nom déclaré du composant corresponde au nom du fichier."
---

# `vue/match-component-file-name`

Exige que le nom déclaré du composant corresponde au nom du fichier.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Le nom du fichier fait partie du contrôle. La comparaison par défaut convertit le nom déclaré du composant en PascalCase ou en kebab-case, puis le compare au nom du fichier sans extension et sans autre transformation. Les fichiers contenant plusieurs définitions de composants ne sont pas contrôlés.

## Mauvais

Le nom déclaré `OtherPanel` ne correspond pas au nom de `MyPanel.vue` sans extension, même après les conversions de casse prises en charge.

`MyPanel.vue`

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "OtherPanel" });
</script>
```

## Bon

Nommez le composant `MyPanel` pour que son nom déclaré corresponde au fichier qui le contient.

`MyPanel.vue`

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "MyPanel" });
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/match-component-file-name": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/match_component_file_name.rs#L21)
