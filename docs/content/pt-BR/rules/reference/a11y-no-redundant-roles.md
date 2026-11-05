---
title: "a11y/no-redundant-roles"
description: "Proibir papéis ARIA redundantes."
---

# `a11y/no-redundant-roles`

Proibir papéis ARIA redundantes.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Sim | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O botão nativo já tem o papel de botão, então `role="button"` repete sua semântica implícita.

```vue annotate="remove:2"
<template>
  <button role="button">Save</button>
</template>
```

## Correto

Remover o papel repetido mantém a semântica de botão fornecida pelo HTML.

```vue annotate="add:2"
<template>
  <button>Save</button>
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
        "a11y/no-redundant-roles": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31)
