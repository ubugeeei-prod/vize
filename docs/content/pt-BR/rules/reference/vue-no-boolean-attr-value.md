---
title: "vue/no-boolean-attr-value"
description: "Proibir valores explícitos em atributos HTML booleanos."
---

# `vue/no-boolean-attr-value`

Proibir valores explícitos em atributos HTML booleanos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Sim | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Os atributos booleanos disabled e checked contêm valores de string redundantes.

```vue annotate="remove:2,3,4"
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

## Correto

A presença de cada atributo booleano expressa o mesmo estado ativado, sem um valor.

```vue annotate="add:2,3,4"
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
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
        "vue/no-boolean-attr-value": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36)
