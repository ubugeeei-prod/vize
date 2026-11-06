---
title: "html/no-consecutive-br"
description: "Proibir elementos &lt;br&gt; consecutivos."
---

# `html/no-consecutive-br`

Proibir elementos &lt;br&gt; consecutivos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Dois elementos de quebra consecutivos criam espaçamento entre blocos dentro de um único parágrafo.

```vue annotate="remove:2"
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

## Correto

Parágrafos separados expressam os dois blocos de conteúdo sem repetir elementos de quebra.

```vue annotate="add:2,3"
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/no-consecutive-br": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras HTML](../html.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30)
