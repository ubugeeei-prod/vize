---
title: "vue/no-undefined-refs"
description: "Proibir referências a variáveis não definidas nos templates."
---

# `vue/no-undefined-refs`

Proibir referências a variáveis não definidas nos templates.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

`globals` lista os nomes que uma etapa de build define para todos os templates, como as importações do `unplugin-auto-import`. Em um projeto Nuxt, o `vize lint` lê as importações automáticas do arquivo gerado `.nuxt/types/imports.d.ts` e as adiciona, portanto elas não precisam ser listadas; até que o `nuxi prepare` gere esse arquivo, a regra é ignorada com um aviso. Os nomes são exatos. As opções não ativam a regra. Camadas posteriores substituem a lista; uma lista vazia remove os nomes herdados.

## Incorreto

O template lê missing, embora o script declare apenas message.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

## Correto

A interpolação lê a variável message existente.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-undefined-refs": "warn"
    },
    "ruleOptions": {
      "vue/no-undefined-refs": {
        "globals": [
          "formatCount"
        ]
      }
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L13)
