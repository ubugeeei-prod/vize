---
title: "html/deprecated-element"
description: "Proibir elementos HTML obsoletos."
---

# `html/deprecated-element`

Proibir elementos HTML obsoletos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O elemento `center` usa um elemento de apresentação HTML obsoleto.

```vue annotate="remove:2"
<template>
  <center>Profile</center>
</template>
```

## Correto

Uma seção e uma classe de estilo substituem o elemento obsoleto, preservando o conteúdo.

```vue annotate="add:2"
<template>
  <section class="profile">Profile</section>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/deprecated-element": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras HTML](../html.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33)
