---
title: "a11y/landmark-roles"
description: "Validar a posição e a unicidade dos papéis de regiões de referência."
---

# `a11y/landmark-roles`

Validar a posição e a unicidade dos papéis de regiões de referência.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Dois elementos `main` declaram regiões de referência principais duplicadas no mesmo template.

```vue annotate="remove:3"
<template>
  <main>Dashboard</main>
  <main>Settings</main>
</template>
```

## Correto

O painel permanece como a região de referência principal; a área de configurações se torna uma região de referência de navegação com nome.

```vue annotate="add:3"
<template>
  <main>Dashboard</main>
  <nav aria-label="Settings">...</nav>
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
        "a11y/landmark-roles": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44)
