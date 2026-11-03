---
title: "vapor/require-vapor-attribute"
description: "Sugerir a adição do atributo vapor a script setup."
---

# `vapor/require-vapor-attribute`

Sugerir a adição do atributo vapor a script setup.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não implementada no lint de SFC | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

> [!WARNING]
> Suporte atual: `no-sfc-finding`. O Vize aceita este ID de regra na configuração, mas o lint de SFC ainda não o reporta. O exemplo correto mostra a convenção pretendida.

Esta regra é um marcador provisório com callback vazio. Adicionar vapor seleciona a compilação Vapor; o linter atual não reporta este ID do catálogo pela ausência desse atributo.

## Incorreto

O bloco script setup não tem o atributo de compilação Vapor. Esta é uma convenção pretendida: a função de retorno atualmente vazia da regra não diagnostica sua ausência.

```vue annotate="remove:1"
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

## Correto

Adicionar vapor seleciona a compilação Vapor. Isso demonstra a correção pretendida e não implica que o linter atual emita esta regra do catálogo.

```vue annotate="add:1"
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

## ID configurado (sem diagnóstico de SFC atualmente)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vapor/require-vapor-attribute": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de Vapor](../vapor.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17)
