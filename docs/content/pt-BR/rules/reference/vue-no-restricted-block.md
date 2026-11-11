---
title: "vue/no-restricted-block"
description: "Proíbe o bloco configurado."
---

# `vue/no-restricted-block`

Proíbe o bloco configurado.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O SFC contém o bloco i18n proibido.

```vue annotate="remove:4"
<template>
  <p>ok</p>
</template>
<i18n lang="json">{}</i18n>
```

## Correto

O SFC mantém apenas o bloco template.

```vue
<template>
  <p>ok</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-block": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-block": [
        "i18n"
      ]
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/blocks.rs#L14)
