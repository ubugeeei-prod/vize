---
title: "vue/no-restricted-v-bind"
description: "Proíbe argumentos restritos de v-bind."
---

# `vue/no-restricted-v-bind`

Proíbe argumentos restritos de v-bind.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

`:v-focus` vincula um argumento de diretiva como atributo.

```vue annotate="remove:2"
<template>
  <input :v-focus="enabled" />
</template>
```

## Correto

`v-focus` usa diretamente a sintaxe de diretiva.

```vue annotate="add:2"
<template>
  <input v-focus="enabled" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-v-bind": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L704)
