---
title: "vue/no-template-lang"
description: "Desencorajar o atributo lang no bloco template."
---

# `vue/no-template-lang`

Desencorajar o atributo lang no bloco template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O template seleciona Pug por meio de lang. Essa é uma convenção pretendida de usar apenas HTML; o processamento atual de SFCs não gera diagnósticos para este ID do catálogo.

```vue annotate="remove:1,2"
<template lang="pug">
p Notice
</template>
```

## Correto

Um template HTML comum omite lang e usa o parágrafo diretamente. Isso ilustra a convenção sem afirmar que há uma ocorrência diagnosticada atualmente em SFCs.

```vue annotate="add:1,2"
<template>
<p>Notice</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-template-lang": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L11)
