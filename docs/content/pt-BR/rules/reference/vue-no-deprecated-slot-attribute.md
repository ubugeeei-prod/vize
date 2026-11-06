---
title: "vue/no-deprecated-slot-attribute"
description: "Proibir o atributo obsoleto `slot`."
---

# `vue/no-deprecated-slot-attribute`

Proibir o atributo obsoleto `slot`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O slot header é selecionado pelo antigo atributo slot.

```vue annotate="remove:3,4"
<template>
  <Foo>
    <template slot="header"><h1>Title</h1></template>
    <div :slot="name">Title</div>
  </Foo>
</template>
```

## Correto

v-slot:header seleciona explicitamente o slot header com a diretiva atual.

```vue annotate="add:3"
<template>
  <Foo>
    <template v-slot:header><h1>Title</h1></template>
  </Foo>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-slot-attribute": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L41)
