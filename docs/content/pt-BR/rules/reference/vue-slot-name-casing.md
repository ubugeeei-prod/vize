---
title: "vue/slot-name-casing"
description: "Exigir a capitalização configurada nos nomes estáticos de pontos de inserção de slot renderizados."
---

# `vue/slot-name-casing`

Exigir a capitalização configurada nos nomes estáticos de pontos de inserção de slot renderizados.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Verifica o nome estático de um ponto de inserção de slot renderizado. A sintaxe de provedor v-slot e #name no componente pai fica fora desta regra.

## Incorreto

O ponto de inserção de slot renderizado usa o nome estático com hífen `my-slot`, enquanto a convenção padrão exige camelCase.

```vue annotate="remove:2"
<template>
<MyCard><slot name="my-slot" /></MyCard>
</template>
```

## Correto

O nome estático do ponto de inserção usa camelCase. A sintaxe de provedor `#name` no componente pai fica fora desta regra.

```vue annotate="add:2"
<template>
<MyCard><slot name="mySlot" /></MyCard>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/slot-name-casing": "warn"
    },
    "ruleOptions": {
      "vue/slot-name-casing": "camelCase"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L30)
