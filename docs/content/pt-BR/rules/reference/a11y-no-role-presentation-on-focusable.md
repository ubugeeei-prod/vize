---
title: "a11y/no-role-presentation-on-focusable"
description: "Proibir role=\"presentation\" ou role=\"none\" em elementos que podem receber foco."
---

# `a11y/no-role-presentation-on-focusable`

Proibir role="presentation" ou role="none" em elementos que podem receber foco.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O link de cobrança, que pode receber foco, solicita role=presentation, o que entra em conflito com seu papel interativo de link; os navegadores devem ignorar essa solicitação de apresentação.

```vue annotate="remove:2"
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

## Correto

Remova a solicitação de apresentação conflitante e use o papel nativo de link e o destino de cobrança.

```vue annotate="add:2"
<template>
  <a href="/billing">Billing</a>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-role-presentation-on-focusable": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19)
