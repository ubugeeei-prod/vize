---
title: "vapor/prefer-static-class"
description: "Preferir class estática a uma vinculação dinâmica de class para literais de string."
---

# `vapor/prefer-static-class`

Preferir class estática a uma vinculação dinâmica de class para literais de string.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Sim | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A vinculação de class avalia uma string constante, embora a classe não mude.

```vue annotate="remove:2"
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

## Correto

Um atributo class estático expressa as mesmas classes do painel sem uma vinculação.

```vue annotate="add:2"
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vapor/prefer-static-class": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de Vapor](../vapor.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L34)
