---
title: "vue/slot-name-casing"
description: "Exigir kebab-case nos slots nomeados usados por v-slot."
---

# `vue/slot-name-casing`

Exigir kebab-case nos slots nomeados usados por v-slot.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O slot nomeado `mySlot` usa camelCase onde a regra exige um nome separado por hífen.

```vue annotate="remove:2"
<template>
<MyCard><template #mySlot>Content</template></MyCard>
</template>
```

## Correto

`#my-slot` usa kebab-case. Renomeie o ponto de inserção do slot correspondente para o mesmo nome.

```vue annotate="add:2"
<template>
<MyCard><template #my-slot>Content</template></MyCard>
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
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L28)
