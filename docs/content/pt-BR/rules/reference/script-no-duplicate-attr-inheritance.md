---
title: "script/no-duplicate-attr-inheritance"
description: "Sinalizar um componente que aplica duas vezes seus atributos repassados."
---

# `script/no-duplicate-attr-inheritance`

Sinalizar um componente que aplica duas vezes seus atributos repassados.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

Os valores explícitos `inheritAttrs: true` repetem o padrão do Vue. Esta regra sinaliza esse literal redundante mesmo quando não há expansão de `$attrs` na raiz.

```vue annotate="remove:1"
<script setup></script>
<template><div><button v-bind="$attrs">Save</button></div></template>
```

## Correto

`inheritAttrs: false` expressa uma desativação efetiva, enquanto o objeto vazio de opções deixa implícita a herança padrão. Nenhuma das formas repete o valor redundante `true`.

```vue annotate="add:1"
<script setup>defineOptions({ inheritAttrs: false })</script>
<template><div><button v-bind="$attrs">Save</button></div></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/no-duplicate-attr-inheritance": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L35)
