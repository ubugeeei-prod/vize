---
title: "a11y/label-has-for"
description: "Exigir controles de formulário associados aos rótulos."
---

# `a11y/label-has-for`

Exigir controles de formulário associados aos rótulos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O rótulo separado não está associado por meio de `for` nem envolve o campo.

```vue annotate="remove:2"
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

## Correto

`for="email"` corresponde ao ID do campo e associa explicitamente os dois elementos.

```vue annotate="add:2"
<template>
  <label for="email">Email</label>
  <input id="email" />
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
        "a11y/label-has-for": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27)
