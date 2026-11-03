---
title: "vue/permitted-contents"
description: "Aplicar as regras do modelo de conteúdo HTML."
---

# `vue/permitted-contents`

Aplicar as regras do modelo de conteúdo HTML.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Os exemplos colocam conteúdo em bloco dentro de p, omitem o corpo da tabela, aninham controles interativos ou colocam uma div diretamente dentro de ul.

```vue annotate="remove:2,3,4,5"
<template>
  <p><div>block in a paragraph</div></p>
  <table><tr><td>row without tbody</td></tr></table>
  <a href="#"><button type="button">nested control</button></a>
  <ul><div>not a list item</div></ul>
</template>
```

## Correto

Os exemplos usam conteúdo inline no parágrafo, um tbody explícito e filhos li. O componente personalizado MyItem não é tratado como um filho nativo conhecido de ul.

```vue annotate="add:2,3,4"
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
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
        "vue/permitted-contents": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56)
