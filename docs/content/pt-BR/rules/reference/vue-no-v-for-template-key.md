---
title: "vue/no-v-for-template-key"
description: "Exige que a key de `template v-for` fique em um filho real para manter compatibilidade com Vue 2."
---

# `vue/no-v-for-template-key`

Exige que a key de `template v-for` fique em um filho real para manter compatibilidade com Vue 2.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Esta é uma regra opcional de compatibilidade com Vue 2. O Vue 2 espera a key em um filho real; o Vue 3 a espera em `<template v-for>`. Para a convenção do Vue 3, use `vue/no-v-for-template-key-on-child`; não ative as duas convenções ao mesmo tempo.

## Incorreto

No contrato do Vue 2, a key em `<template v-for>` é ignorada porque o template não renderiza um elemento real.

```vue annotate="remove:1"
<template><ul><template v-for="item in items" :key="item.id"><li>{{ item.label }}</li></template></ul></template>
```

## Correto

Coloque a key do item no filho real `li` repetido, conforme a convenção do Vue 2 verificada aqui.

```vue annotate="add:1"
<template><ul><template v-for="item in items"><li :key="item.id">{{ item.label }}</li></template></ul></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-for-template-key": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key.rs#L33)
