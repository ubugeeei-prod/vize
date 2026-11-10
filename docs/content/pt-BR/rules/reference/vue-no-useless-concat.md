---
title: "vue/no-useless-concat"
description: "Proíbe concatenação inútil de literais de string."
---

# `vue/no-useless-concat`

Proíbe concatenação inútil de literais de string.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Dois literais são concatenados em tempo de execução.

```vue annotate="remove:2"
<template>
  <p>{{ "hello" + " world" }}</p>
</template>
```

## Correto

Um único literal representa o mesmo texto.

```vue annotate="add:2"
<template>
  <p>{{ "hello world" }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-useless-concat": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_useless_concat.rs#L12)
