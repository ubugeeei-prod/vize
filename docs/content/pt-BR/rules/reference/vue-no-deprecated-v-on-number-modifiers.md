---
title: "vue/no-deprecated-v-on-number-modifiers"
description: "Proibir modificadores numéricos obsoletos de `keyCode` em `v-on`."
---

# `vue/no-deprecated-v-on-number-modifiers`

Proibir modificadores numéricos obsoletos de `keyCode` em `v-on`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Sim | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Seleção de modificadores numéricos e correções

O primeiro modificador numérico cujo nome começa com um inteiro decimal determina o diagnóstico. `@keyup.13.27="submit"` reporta apenas `13` e oferece `@keyup.enter.27="submit"`; outra passagem de correção pode substituir `27` por `escape`. `@keyup.13suffix="submit"` também se torna `@keyup.enter="submit"`. Os valores de 0 a 9 são isentos, portanto `@keyup.0.13="submit"` não gera diagnóstico desta regra. Valores desconhecidos, como `999`, são reportados sem correção. A isenção existente para Vue 2 e petite-vue continua em vigor.

## Incorreto

Os manipuladores de teclado identificam as teclas pelos códigos numéricos removidos 13 e 27.

```vue annotate="remove:2,3,4"
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

## Correto

Os manipuladores usam os modificadores de tecla nomeados enter e esc.

```vue annotate="add:2,3"
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-v-on-number-modifiers": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L45)
