---
title: "vue/no-bare-strings-in-template"
description: "Proibir texto legível por pessoas diretamente no template quando ele deve ser internacionalizado."
---

# `vue/no-bare-strings-in-template`

Proibir texto legível por pessoas diretamente no template quando ele deve ser internacionalizado.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Quando presente, allowlist substitui os padrões de pontuação do eslint-plugin-vue. As entradas são substrings literais ou valores /pattern/flags, removidos antes da verificação do texto restante. Omita allowlist para manter os padrões; [] os remove. As opções não ativam a regra.

## Incorreto

O texto visível incorpora uma string sem tradução diretamente no template.

```vue annotate="remove:2"
<template>
<p>Translate me</p>
</template>
```

## Correto

A allowlist configurada aceita o rótulo de tecla único `W`.

```vue annotate="add:2"
<template>
<kbd>W</kbd>
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
    },
    "ruleOptions": {
      "vue/no-bare-strings-in-template": {
        "allowlist": [
          "/^[DRWXYmvx]$/u"
        ]
      }
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L51)
