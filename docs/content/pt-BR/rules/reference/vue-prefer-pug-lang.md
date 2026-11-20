---
title: "vue/prefer-pug-lang"
description: "Preferir lang=\"pug\" à linguagem de template legada jade 1.x."
---

# `vue/prefer-pug-lang`

Preferir lang="pug" à linguagem de template legada jade 1.x.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Sim | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O template nomeia jade, que o Vue renderiza com o pacote legado jade 1.x, e usa o each com hífen exclusivo do jade.

```vue annotate="remove:1,3"
<template lang="jade">
ul
  - each item in [1, 2]
    li= item
</template>
```

## Correto

O template nomeia pug e escreve o laço com a palavra-chave each do pug 3.

```vue annotate="add:1,3"
<template lang="pug">
ul
  each item in [1, 2]
    li= item
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-pug-lang": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prefer_pug_lang.rs#L42)
