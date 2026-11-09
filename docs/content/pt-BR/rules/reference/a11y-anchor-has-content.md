---
title: "a11y/anchor-has-content"
description: "Exigir conteúdo acessível nos elementos de âncora."
---

# `a11y/anchor-has-content`

Exigir conteúdo acessível nos elementos de âncora.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Liste os componentes cuja subárvore renderizada sempre fornece conteúdo acessível ao link. As grafias PascalCase e kebab-case são equivalentes. As opções não ativam a regra; camadas posteriores substituem a lista inteira, e [] remove os nomes herdados.

## Incorreto

O link `/app` não tem texto nem outro conteúdo que lhe dê um nome, então seu destino não tem uma descrição acessível.

```vue annotate="remove:2"
<template>
<a href="/app"></a>
</template>
```

## Correto

O texto `alt` da imagem nativa fornece conteúdo acessível ao link com o mesmo destino.

```vue annotate="add:2"
<template>
<a href="/app"><img src="/badge.png" alt="Get the app" /></a>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/anchor-has-content": "warn"
    },
    "ruleOptions": {
      "a11y/anchor-has-content": {
        "accessibleChildren": [
          "RetryImage"
        ]
      }
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L24)
