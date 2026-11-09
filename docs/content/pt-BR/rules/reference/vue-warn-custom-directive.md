---
title: "vue/warn-custom-directive"
description: "Alertar sobre diretivas personalizadas que precisam de registro."
---

# `vue/warn-custom-directive`

Alertar sobre diretivas personalizadas que precisam de registro.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Liste os nomes exatos das diretivas registradas globalmente, sem o prefixo v-. Bindings vName locais de script-setup continuam sendo descobertos automaticamente. As opções não ativam a regra; camadas posteriores substituem a lista inteira, e [] remove os nomes herdados.

## Incorreto

`v-focus` exige uma implementação de diretiva específica do projeto, que esta convenção opcional reporta.

```vue annotate="remove:2"
<template>
<button type="button" v-focus>Focus</button>
</template>
```

## Correto

`v-show` é uma diretiva nativa do Vue e não precisa de registro no projeto.

```vue annotate="add:2"
<template>
<button type="button" v-show="visible">Copy</button>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/warn-custom-directive": "warn"
    },
    "ruleOptions": {
      "vue/warn-custom-directive": {
        "ignore": [
          "hover-tooltip"
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44)
