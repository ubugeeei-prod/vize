---
title: "html/require-datetime"
description: "Exigir o atributo datetime no elemento &lt;time&gt;."
---

# `html/require-datetime`

Exigir o atributo datetime no elemento &lt;time&gt;.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O elemento time contém uma data legível por pessoas, mas não tem um valor datetime legível por máquinas.

```vue annotate="remove:2"
<template>
  <time>May 13, 2026</time>
</template>
```

## Correto

`datetime="2026-05-13"` fornece a data correspondente em um formato legível por máquinas.

```vue annotate="add:2"
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
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
        "html/require-datetime": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras HTML](../html.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34)
