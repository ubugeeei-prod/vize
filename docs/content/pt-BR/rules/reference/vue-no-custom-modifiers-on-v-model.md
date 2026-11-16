---
title: "vue/no-custom-modifiers-on-v-model"
description: "Proíbe modificadores personalizados no `v-model` de componentes para manter compatibilidade com Vue 2."
---

# `vue/no-custom-modifiers-on-v-model`

Proíbe modificadores personalizados no `v-model` de componentes para manter compatibilidade com Vue 2.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Esta regra opcional aplica um contrato de compatibilidade com Vue 2 ao `v-model` de componentes. Apenas os modificadores `lazy`, `number` e `trim` são aceitos. Componentes do Vue 3 podem implementar modificadores personalizados, por isso as predefinições do Vue 3 não ativam esta restrição. Elementos nativos ficam fora desta verificação.

## Incorreto

O modificador `.capitalize` do componente é personalizado e viola esta restrição de compatibilidade com Vue 2.

```vue annotate="remove:1"
<template><MyField v-model.capitalize="value" /></template>
```

## Correto

Use o contrato normal de `v-model` do componente, sem um modificador personalizado; implemente explicitamente a capitalização, se necessária.

```vue annotate="add:1"
<template><MyField v-model="value" /></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-custom-modifiers-on-v-model": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_custom_modifiers_on_v_model.rs#L23)
