---
title: "vue/no-v-for-template-key-on-child"
description: "Proibir `key` no filho de um `<template v-for>`."
---

# `vue/no-v-for-template-key-on-child`

Proibir `key` no filho de um `<template v-for>`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O parágrafo filho tem a key, enquanto a própria iteração template não tem chave.

```vue annotate="remove:2"
<template>
<template v-for="item in items"><p :key="item.id">{{ item.name }}</p></template>
</template>
```

## Correto

A key passa para template v-for, identificando o fragmento repetido completo.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><p>{{ item.name }}</p></template>
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
        "vue/no-v-for-template-key-on-child": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30)
