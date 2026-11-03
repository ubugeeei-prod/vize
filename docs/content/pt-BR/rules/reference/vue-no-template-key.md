---
title: "vue/no-template-key"
description: "Proibir o atributo `key` em `<template>`."
---

# `vue/no-template-key`

Proibir o atributo `key` em `<template>`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Um invólucro template sem laço tem uma key, embora não seja o limite da iteração com chave.

```vue annotate="remove:2"
<template>
<template :key="section"><div>Details</div></template>
</template>
```

## Correto

A key pertence a uma iteração template v-for, na qual identifica cada fragmento repetido.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
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
        "vue/no-template-key": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31)
