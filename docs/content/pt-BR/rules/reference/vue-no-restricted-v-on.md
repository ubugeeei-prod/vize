---
title: "vue/no-restricted-v-on"
description: "Proíbe o argumento de v-on configurado."
---

# `vue/no-restricted-v-on`

Proíbe o argumento de v-on configurado.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O botão escuta o evento proibido legacy-click.

```vue annotate="remove:2"
<template>
  <button type="button" @legacy-click="save">x</button>
</template>
```

## Correto

O botão escuta click.

```vue annotate="add:2"
<template>
  <button type="button" @click="save">x</button>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-v-on": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-v-on": [
        "legacy-click"
      ]
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_v_on.rs#L13)
