---
title: "vue/require-component-registration"
description: "Exigir importação ou registro explícito de componentes."
---

# `vue/require-component-registration`

Exigir importação ou registro explícito de componentes.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Liste nomes explícitos de componentes fornecidos por plugins da aplicação ou pelo previewSetup do Musea. Grafias PascalCase e kebab-case são aceitas; globals contém nomes, não expressões regulares. ignorePatterns recebe expressões regulares, como vue/no-undef-components do eslint-plugin-vue. Um atributo `is` estático nomeia um componente em `<component>` ou com o prefixo `vue:`; em outro elemento, é um elemento nativo personalizado e não é verificado. As opções não ativam a regra. Camadas posteriores substituem a lista; uma lista vazia remove os nomes herdados.

## Incorreto

`MissingWidget` não está registrado nem incluído na lista configurada de componentes globais permitidos.

```vue annotate="remove:2"
<template>
<MissingWidget />
</template>
```

## Correto

`MyButton` consta na opção `globals` do exemplo. Essa opção isenta um componente global conhecido; ela não o registra nem o importa.

```vue annotate="add:2"
<template>
<MyButton />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-component-registration": "warn"
    },
    "ruleOptions": {
      "vue/require-component-registration": {
        "globals": [
          "MyButton",
          "MyIcon"
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L59)
