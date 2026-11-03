---
title: "vue/no-root-v-if"
description: "Proibir v-if no único elemento raiz de um template."
---

# `vue/no-root-v-if`

Proibir v-if no único elemento raiz de um template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A própria raiz do componente aparece e desaparece sob v-if.

```vue annotate="remove:2"
<template>
  <div v-if="show">content</div>
</template>
```

## Correto

Uma div externa estável permanece como raiz, enquanto o parágrafo aninhado recebe a condição de visibilidade.

```vue annotate="add:2,3,4"
<template>
  <div>
    <p v-if="show">content</p>
  </div>
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
        "vue/no-root-v-if": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40)
