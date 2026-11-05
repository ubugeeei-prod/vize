---
title: "a11y/no-autofocus"
description: "Proibir o uso do atributo autofocus."
---

# `a11y/no-autofocus`

Proibir o uso do atributo autofocus.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O campo solicita foco automático quando aparece.

```vue annotate="remove:2"
<template>
  <input autofocus name="query" />
</template>
```

## Correto

Remover `autofocus` evita essa solicitação de foco automático e mantém o campo de consulta.

```vue annotate="add:2"
<template>
  <input name="query" />
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
        "a11y/no-autofocus": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L21)
