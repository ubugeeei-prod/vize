---
title: "vue/no-deprecated-slot-scope-attribute"
description: "Proibir o atributo obsoleto `slot-scope`."
---

# `vue/no-deprecated-slot-scope-attribute`

Proibir o atributo obsoleto `slot-scope`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O template recebe as props do slot pelo atributo obsoleto slot-scope.

```vue annotate="remove:2"
<template>
<Card><template slot-scope="props">{{ props.name }}</template></Card>
</template>
```

## Correto

A diretiva #default recebe essas props sem slot-scope.

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-slot-scope-attribute": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33)
