---
title: "vue/no-deprecated-router-link-tag-prop"
description: "Proibir a prop `tag` em &lt;router-link&gt;."
---

# `vue/no-deprecated-router-link-tag-prop`

Proibir a prop `tag` em &lt;router-link&gt;.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

RouterLink usa a prop tag removida para solicitar um elemento button.

```vue annotate="remove:2"
<template>
  <router-link to="/home" tag="button">Home</router-link>
</template>
```

## Correto

O slot fornece navigate a um botão escrito explicitamente no template.

```vue annotate="add:2,3,4"
<template>
  <router-link to="/home" v-slot="{ navigate }">
    <button @click="navigate">Home</button>
  </router-link>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-router-link-tag-prop": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L38)
