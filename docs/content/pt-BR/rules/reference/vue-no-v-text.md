---
title: "vue/no-v-text"
description: "Proibir a diretiva v-text; preferir interpolação com chaves duplas."
---

# `vue/no-v-text`

Proibir a diretiva v-text; preferir interpolação com chaves duplas.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O conteúdo da div é fornecido pela diretiva v-text.

```vue annotate="remove:2"
<template>
<div v-text="message"></div>
</template>
```

## Correto

A interpolação com chaves duplas expressa a mesma vinculação de texto diretamente no conteúdo do elemento.

```vue annotate="add:2"
<template>
<div>{{ message }}</div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-v-text": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L32)
