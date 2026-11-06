---
title: "html/no-empty-palpable-content"
description: "Proibir elementos vazios que esperam conteúdo visível."
---

# `html/no-empty-palpable-content`

Proibir elementos vazios que esperam conteúdo visível.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O parágrafo, o item de lista e a célula da tabela têm conteúdo perceptível vazio.

```vue annotate="remove:2,3,4"
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

## Correto

O texto preenche o parágrafo, a interpolação fornece o conteúdo do item de lista e aria-label dá um nome explícito à célula que, de outra forma, estaria vazia.

```vue annotate="add:2,3,4"
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/no-empty-palpable-content": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras HTML](../html.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32)
