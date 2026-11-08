---
title: "html/no-dupe-style-properties"
description: "Proibir propriedades duplicadas em atributos de estilo em linha."
---

# `html/no-dupe-style-properties`

Proibir propriedades duplicadas em atributos de estilo em linha.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Cada estilo estático repete uma propriedade; `margin` e `MARGIN` também contam como a mesma propriedade.

```vue annotate="remove:2,3"
<template>
  <div style="color: red; color: blue">text</div>
  <div style="margin: 0; MARGIN: 1px">text</div>
</template>
```

## Correto

O estilo estático usa propriedades distintas de cor e plano de fundo. Vinculações dinâmicas de estilo ficam fora desta verificação de atributos estáticos.

```vue annotate="add:2,3"
<template>
  <div style="color: red; background: blue">text</div>
  <div :style="{ color: a, color: b }">text</div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/no-dupe-style-properties": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras HTML](../html.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L39)
