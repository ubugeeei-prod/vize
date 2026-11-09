---
title: "a11y/label-has-for"
description: "Exigir controles de formulário associados aos rótulos."
---

# `a11y/label-has-for`

Exigir controles de formulário associados aos rótulos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Liste os componentes que renderizam um controle de formulário nativo e satisfazem o aninhamento do label. As grafias PascalCase e kebab-case são equivalentes. As opções não ativam a regra; camadas posteriores substituem a lista inteira, e [] remove os nomes herdados.

## Incorreto

O rótulo separado não está associado por meio de `for` nem envolve o campo.

```vue annotate="remove:2"
<template>
<label>Name</label>
</template>
```

## Correto

O input nativo aninhado fica implicitamente associado ao label que o envolve.

```vue annotate="add:2"
<template>
<label>Name <input v-model="name" /></label>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/label-has-for": "warn"
    },
    "ruleOptions": {
      "a11y/label-has-for": {
        "controlComponents": [
          "CInputField"
        ]
      }
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L36)
