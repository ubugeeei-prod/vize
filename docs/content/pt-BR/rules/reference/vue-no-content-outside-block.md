---
title: "vue/no-content-outside-block"
description: "Proíbe conteúdo fora dos blocos superiores do SFC."
---

# `vue/no-content-outside-block`

Proíbe conteúdo fora dos blocos superiores do SFC.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O texto inicial está fora de todos os blocos do SFC.

```vue annotate="remove:1,2"
unexpected
<template><div /></template>
```

## Correto

Todo o conteúdo está dentro do bloco template.

```vue annotate="add:1"
<template><div>expected</div></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-content-outside-block": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/content.rs#L8)
