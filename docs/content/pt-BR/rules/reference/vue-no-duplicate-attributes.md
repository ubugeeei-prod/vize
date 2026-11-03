---
title: "vue/no-duplicate-attributes"
description: "Proibir atributos duplicados no mesmo elemento."
---

# `vue/no-duplicate-attributes`

Proibir atributos duplicados no mesmo elemento.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O mesmo botão declara class duas vezes, em vez de usar um único valor combinado de class.

```vue annotate="remove:2"
<template>
  <button class="primary" class="large">Save</button>
</template>
```

## Correto

Os dois nomes de classe aparecem em um único atributo class.

```vue annotate="add:2"
<template>
  <button class="primary large">Save</button>
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
        "vue/no-duplicate-attributes": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31)
