---
title: "vue/no-static-inline-styles"
description: "Proibir atributos estáticos de estilo inline."
---

# `vue/no-static-inline-styles`

Proibir atributos estáticos de estilo inline.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O parágrafo contém a declaração de cor constante no atributo style.

```vue annotate="remove:1,2,3"
<template>
<p style="color: red">Notice</p>
</template>
```

## Correto

Uma classe notice e uma folha de estilo com escopo mantêm a cor constante fora do atributo no template.

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-static-inline-styles": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L21)
