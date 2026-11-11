---
title: "vue/block-lang"
description: "Restringe o lang do bloco SFC ao idioma configurado."
---

# `vue/block-lang`

Restringe o lang do bloco SFC ao idioma configurado.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O bloco script omite o atributo lang configurado.

```vue annotate="remove:1"
<script setup>
const label = "a";
</script>
<template>
  <p>{{ label }}</p>
</template>
```

## Correto

O bloco script define lang como o idioma configurado.

```vue annotate="add:1"
<script setup lang="ts">
const label = "a";
</script>
<template>
  <p>{{ label }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/block-lang": "warn"
    },
    "ruleOptions": {
      "vue/block-lang": {
        "script": {
          "lang": "ts",
          "allowNoLang": false
        }
      }
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/block_lang.rs#L12)
