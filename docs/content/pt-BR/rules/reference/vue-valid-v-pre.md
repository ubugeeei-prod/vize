---
title: "vue/valid-v-pre"
description: "Exigir diretivas `v-pre` válidas."
---

# `vue/valid-v-pre`

Exigir diretivas `v-pre` válidas.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

`v-pre` tem um argumento, modificador ou valor, embora esta diretiva seja um marcador sem valor que ignora a compilação.

```vue annotate="remove:2,3,4"
<template>
<div v-pre:arg></div>
<div v-pre.mod></div>
<div v-pre="foo"></div>
</template>
```

## Correto

`v-pre` sozinho deixa a subárvore sem compilar, então suas interpolações são exibidas como escritas.

```vue annotate="add:2"
<template>
<div v-pre>{{ raw }}</div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-pre": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_pre.rs#L30)
