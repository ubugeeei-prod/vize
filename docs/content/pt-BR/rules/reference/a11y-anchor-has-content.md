---
title: "a11y/anchor-has-content"
description: "Exigir conteúdo acessível nos elementos de âncora."
---

# `a11y/anchor-has-content`

Exigir conteúdo acessível nos elementos de âncora.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O link `/settings` não tem texto nem outro conteúdo que lhe dê um nome, então seu destino não tem uma descrição acessível.

```vue annotate="remove:2"
<template>
  <a href="/settings"></a>
</template>
```

## Correto

O texto visível `Settings` fornece conteúdo para o link com o mesmo destino.

```vue annotate="add:2"
<template>
  <a href="/settings">Settings</a>
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
        "a11y/anchor-has-content": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L20)
