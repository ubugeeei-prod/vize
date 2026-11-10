---
title: "vue/no-console"
description: "Proíbe acesso ao console em expressões do template."
---

# `vue/no-console`

Proíbe acesso ao console em expressões do template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A expressão do evento chama `console.log`.

```vue annotate="remove:2"
<template>
  <button @click="console.log(value)">Save</button>
</template>
```

## Correto

O template delega o log a uma referência do script.

```vue annotate="add:2"
<template>
  <button @click="logValue">Save</button>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-console": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_console.rs#L12)
