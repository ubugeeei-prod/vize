---
title: "vue/no-bare-strings-in-template"
description: "Proibir texto legível por pessoas diretamente no template quando ele deve ser internacionalizado."
---

# `vue/no-bare-strings-in-template`

Proibir texto legível por pessoas diretamente no template quando ele deve ser internacionalizado.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O texto visível e os atributos de identificação incorporam strings sem tradução diretamente no template.

```vue annotate="remove:2,3,4,5"
<template>
<div>hello</div>
<img alt="a cat" />
<input placeholder="Search" />
<button title="Close">x</button>
</template>
```

## Correto

O conteúdo traduzível chama $t; os exemplos com pontuação e apenas números são exceções permitidas.

```vue annotate="add:2,3,4,5"
<template>
<div>{{ $t('hello') }}</div>
<img :alt="$t('cat')" />
<div>-</div>
<button :title="$t('close')">{{ $t('x') }}</button>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-bare-strings-in-template": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L50)
