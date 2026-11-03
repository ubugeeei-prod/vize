---
title: "vue/no-inline-style"
description: "Desencorajar o uso de atributos de estilo inline."
---

# `vue/no-inline-style`

Desencorajar o uso de atributos de estilo inline.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O atributo style estático incorpora a declaração de cor no elemento.

```vue annotate="remove:2"
<template>
  <div style="color: red">Text</div>
</template>
```

## Correto

Classes expressam a cor fixa; a largura dependente de ratio permanece como uma vinculação dinâmica de estilo, fora da verificação de atributos estáticos.

```vue annotate="add:2,3,4"
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
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
        "vue/no-inline-style": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33)
