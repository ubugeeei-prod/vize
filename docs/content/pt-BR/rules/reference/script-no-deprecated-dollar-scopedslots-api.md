---
title: "script/no-deprecated-dollar-scopedslots-api"
description: "Proibir a propriedade de instância $scopedSlots removida no Vue 3 (usar $slots)."
---

# `script/no-deprecated-dollar-scopedslots-api`

Proibir a propriedade de instância $scopedSlots removida no Vue 3 (usar $slots).

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`this.$scopedSlots`, `ctx.$scopedSlots` e a referência direta a `$scopedSlots` usam a API de slots com escopo do Vue 2 removida no Vue 3.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

## Correto

Substituir `$scopedSlots` por `$slots` usa a API unificada de slots. O exemplo remove a grafia obsoleta em vez de estabelecer um contexto de setup para os objetos receptores.

```vue annotate="add:2,3,4"
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-dollar-scopedslots-api": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L45)
