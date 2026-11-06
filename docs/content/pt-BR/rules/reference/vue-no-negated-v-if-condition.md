---
title: "vue/no-negated-v-if-condition"
description: "Proibir uma condição negada em v-if quando a cadeia tiver v-else."
---

# `vue/no-negated-v-if-condition`

Proibir uma condição negada em v-if quando a cadeia tiver v-else.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Os ramos emparelhados v-if e v-else começam com uma condição negada.

```vue
<template>
<div v-if="!ok">A</div>
<div v-else>B</div>
</template>
```

## Correto

Uma condição positiva ok vem primeiro; ao inverter uma condição, coloque primeiro o ramo originalmente oposto. Um v-if negado isolado e comparações !== continuam permitidos.

```vue annotate="add:2,3,4,6,7"
<template>
<div v-if="ok">B</div>
<div v-else>A</div>

<div v-if="!ok">A</div>

<div v-if="a !== b">A</div>
<div v-else>B</div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-negated-v-if-condition": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L37)
