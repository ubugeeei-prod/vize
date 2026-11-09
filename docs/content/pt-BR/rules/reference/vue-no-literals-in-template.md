---
title: "vue/no-literals-in-template"
description: "Proíbe literais de objeto, array e função em v-bind."
---

# `vue/no-literals-in-template`

Proíbe literais de objeto, array e função em v-bind.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O literal de objeto cria um novo valor durante a renderização.

```vue annotate="remove:2"
<template>
  <Widget :options="{ eager: true }" />
</template>
```

## Correto

Uma referência estável do script fornece o objeto de opções.

```vue annotate="add:2"
<template>
  <Widget :options="options" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-literals-in-template": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L657)
