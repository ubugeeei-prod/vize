---
title: "ecosystem/vue-test-utils-no-html-snapshot"
description: "Éviter les instantanés de wrapper.html() dans les tests Vue Test Utils."
---

# `ecosystem/vue-test-utils-no-html-snapshot`

Éviter les instantanés de wrapper.html() dans les tests Vue Test Utils.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

L’assertion capture l’ensemble du HTML du wrapper au lieu de vérifier le comportement attendu.

```vue annotate="remove:2"
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

## Bon

L’assertion vérifie que le texte affiché contient Saved.

```vue annotate="add:2"
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
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
        "ecosystem/vue-test-utils-no-html-snapshot": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de l’écosystème et de Nuxt](../ecosystem.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15)
