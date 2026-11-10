---
title: "vue/attribute-hyphenation"
description: "Aplicar um padrão de nomes de atributos em componentes personalizados."
---

# `vue/attribute-hyphenation`

Aplicar um padrão de nomes de atributos em componentes personalizados.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Sim | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O atributo do componente usa a grafia camelCase firstName.

```vue annotate="remove:2"
<template>
<UserCard firstName="Ada" />
</template>
```

## Correto

A grafia first-name segue a convenção configurada de atributos de componente separados por hífen.

```vue annotate="add:2"
<template>
<UserCard first-name="Ada" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/attribute-hyphenation": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35)
