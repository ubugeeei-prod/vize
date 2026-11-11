---
title: "vue/no-restricted-static-attribute"
description: "Proíbe o atributo estático configurado."
---

# `vue/no-restricted-static-attribute`

Proíbe o atributo estático configurado.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O botão tem o atributo proibido data-legacy.

```vue annotate="remove:2"
<template>
  <button type="button" data-legacy="x">x</button>
</template>
```

## Correto

O botão mantém apenas type.

```vue annotate="add:2"
<template>
  <button type="button">x</button>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-static-attribute": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-static-attribute": [
        "data-legacy"
      ]
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_static_attribute.rs#L13)
