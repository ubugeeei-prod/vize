---
title: "a11y/form-control-has-label"
description: "Exigir rótulos associados aos controles de formulário."
---

# `a11y/form-control-has-label`

Exigir rótulos associados aos controles de formulário.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O campo de pesquisa não tem um rótulo que identifique o que o usuário deve inserir.

```vue annotate="remove:2"
<template>
  <input type="search" />
</template>
```

## Correto

Envolver o campo em um rótulo associa o texto visível `Search` ao controle.

```vue annotate="add:2,3,4,5"
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/form-control-has-label": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L22)
