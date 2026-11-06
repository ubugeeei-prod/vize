---
title: "vue/no-multiple-template-root"
description: "Proibir vários nós raiz em um template."
---

# `vue/no-multiple-template-root`

Proibir vários nós raiz em um template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Ative apenas quando houver um contrato de raiz única. O Vue 3 normalmente suporta fragmentos.

## Incorreto

A convenção opcional de raiz única encontra dois parágrafos irmãos na raiz do template.

```vue annotate="remove:2,3"
<template>
<p>First</p>
<p>Second</p>
</template>
```

## Correto

Uma section envolve os parágrafos em uma única raiz; ative essa convenção apenas quando houver um contrato de raiz única.

```vue annotate="add:2"
<template>
<section><p>First</p><p>Second</p></section>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-multiple-template-root": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L23)
