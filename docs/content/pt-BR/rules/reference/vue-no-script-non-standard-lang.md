---
title: "vue/no-script-non-standard-lang"
description: "Desencorajar valores não padronizados de lang em scripts."
---

# `vue/no-script-non-standard-lang`

Desencorajar valores não padronizados de lang em scripts.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não implementada no lint de SFC | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

> [!WARNING]
> Suporte atual: `no-sfc-finding`. O Vize aceita este ID de regra na configuração, mas o lint de SFC ainda não o reporta. O exemplo correto mostra a convenção pretendida.

Esta entrada do catálogo atualmente não emite uma ocorrência específica da regra pelo lint de SFCs. O par Bad/Good descreve a convenção pretendida, e não uma ocorrência executável. Ativar o ID não fornece a verificação de SFC ausente.

## Incorreto

O script usa sintaxe CoffeeScript com lang=coffee. O processamento atual de SFCs não emite esta regra do catálogo para essa linguagem.

```vue annotate="remove:1,2"
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

## Correto

O script usa uma declaração TypeScript comum com lang=ts, ilustrando a convenção de linguagem pretendida.

```vue annotate="add:1,2"
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

## ID configurado (sem diagnóstico de SFC atualmente)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-script-non-standard-lang": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44)
