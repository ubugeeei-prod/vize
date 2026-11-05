---
title: "a11y/no-static-element-interactions"
description: "Proibir manipuladores de eventos em elementos estáticos."
---

# `a11y/no-static-element-interactions`

Proibir manipuladores de eventos em elementos estáticos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Uma seção estática recebe uma ação da tecla Enter sem ter um papel interativo.

```vue annotate="remove:2"
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

## Correto

Um botão nativo executa a mesma ação usando um elemento interativo adequado.

```vue annotate="add:2"
<template>
  <button type="button" @keydown.enter="select">Select</button>
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
        "a11y/no-static-element-interactions": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L36)
