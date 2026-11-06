---
title: "a11y/tabindex-no-positive"
description: "Proibir valores positivos de tabindex."
---

# `a11y/tabindex-no-positive`

Proibir valores positivos de tabindex.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Um tabindex positivo de 3 cria uma ordem de foco personalizada antes dos controles comuns.

```vue annotate="remove:2"
<template>
  <button tabindex="3">Save</button>
</template>
```

## Correto

O botão usa sua ordem de foco nativa sem um tabindex positivo.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/tabindex-no-positive": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16)
