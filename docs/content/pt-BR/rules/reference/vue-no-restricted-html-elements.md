---
title: "vue/no-restricted-html-elements"
description: "Proíbe o elemento configurado."
---

# `vue/no-restricted-html-elements`

Proíbe o elemento configurado.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O template usa o marquee proibido.

```vue annotate="remove:2"
<template>
  <marquee>x</marquee>
</template>
```

## Correto

O template usa p.

```vue annotate="add:2"
<template>
  <p>x</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-html-elements": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-html-elements": [
        "marquee"
      ]
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_html.rs#L13)
