---
title: "vue/comment-directive"
description: "Aceita comentários de controle de diagnósticos em `<template>`."
---

# `vue/comment-directive`

Aceita comentários de controle de diagnósticos em `<template>`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Comentários de supressão não utilizados no template só são reportados quando `reportUnusedDisableDirectives: true` está ativado. O exemplo remove um comentário sem uso; comentários que realmente suprimem um diagnóstico continuam úteis. Nomes de regras listados só contam quando o Vize executa essas regras.

## Incorreto

O comentário que desativa todas as regras não suprime nenhum diagnóstico no `div` seguinte, por isso a verificação de diretivas não utilizadas o reporta.

```vue annotate="remove:2"
<template>
  <!-- eslint-disable-next-line -->
  <div />
</template>
```

## Correto

Remova o comentário desnecessário e mantenha o elemento, deixando as verificações de lint ativas nessa linha.

```vue
<template>
  <div />
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
        "vue/comment-directive": "error"
      },
      "reportUnusedDisableDirectives": true
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/comment_directive.rs#L33)
