---
title: "vue/object-shorthand"
description: "Exige propriedades abreviadas nos objetos do template."
---

# `vue/object-shorthand`

Exige propriedades abreviadas nos objetos do template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O objeto repete `value` como chave e valor.

```vue annotate="remove:2"
<template>
  <Widget :options="{ value: value }" />
</template>
```

## Correto

A abreviação preserva a referência sem repetição.

```vue annotate="add:2"
<template>
  <Widget :options="{ value }" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/object-shorthand": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L138)
