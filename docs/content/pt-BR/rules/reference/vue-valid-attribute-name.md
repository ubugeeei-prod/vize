---
title: "vue/valid-attribute-name"
description: "Exigir nomes de atributos válidos."
---

# `vue/valid-attribute-name`

Exigir nomes de atributos válidos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Diagnóstico do exemplo incorreto: `parser/template`

A grafia malformada de um atributo é diagnosticada por parser/template antes que esta regra defensiva veja um atributo. Portanto, Bad reporta parser/template; não promete uma ocorrência separada de vue/valid-attribute-name.

## Incorreto

A aspa dentro de `my"attr` torna o nome do atributo malformado. Este exemplo produz o diagnóstico `parser/template` do parser, sem prometer um diagnóstico separado da regra.

```vue annotate="remove:2"
<template>
<div my"attr="value"></div>
</template>
```

## Correto

`my-attr` é um nome de atributo bem formado, então o parser do template pode ler o atributo e seu valor.

```vue annotate="add:2"
<template>
<div my-attr="value"></div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-attribute-name": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L28)
