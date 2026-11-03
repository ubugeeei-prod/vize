---
title: "a11y/interactive-supports-focus"
description: "Exigir que elementos com papéis interativos possam receber foco."
---

# `a11y/interactive-supports-focus`

Exigir que elementos com papéis interativos possam receber foco.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Atribuir a um `span` o papel de botão e um manipulador de clique não permite que o elemento receba foco pelo teclado.

```vue annotate="remove:2"
<template>
  <span role="button" @click="open">Open</span>
</template>
```

## Correto

O botão nativo pode receber foco e mantém a mesma ação `open`.

```vue annotate="add:2"
<template>
  <button type="button" @click="open">Open</button>
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
        "a11y/interactive-supports-focus": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L39)
