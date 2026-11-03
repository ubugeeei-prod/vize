---
title: "script/valid-define-options"
description: "Imposer une utilisation valide de defineOptions() (un seul argument objet, sans props/emits/expose/slots)."
---

# `script/valid-define-options`

Imposer une utilisation valide de defineOptions() (un seul argument objet, sans props/emits/expose/slots).

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Le premier appel place la déclaration dédiée `props` dans `defineOptions` ; les appels suivants répètent également la macro et incluent un argument qui n’est pas un objet. Ils illustrent les contraintes sur les formes interdites et les appels répétés.

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
defineOptions({ props: ['foo'] })   // use defineProps instead
defineOptions({ name: 'Foo' })
defineOptions({ name: 'Bar' })      // duplicate call
defineOptions('Foo')                // not an object literal
</script>
```

## Bon

Un seul appel `defineOptions` reçoit un objet contenant uniquement les options ordinaires prises en charge `name` et `inheritAttrs`.

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: 'Foo', inheritAttrs: false })
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
        "script/valid-define-options": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41)
