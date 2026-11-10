---
title: "vue/use-unique-element-ids"
description: "Exigir IDs de elementos únicos por meio de useId(), em vez de literais estáticos."
---

# `vue/use-unique-element-ids`

Exigir IDs de elementos únicos por meio de useId(), em vez de literais estáticos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O ID literal `email` é reutilizado por todas as instâncias deste componente, o que pode direcionar seu rótulo ao elemento errado quando várias instâncias são renderizadas.

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

## Correto

`useId()` produz o `emailId` da instância; vincule o mesmo valor ao `for` do rótulo e ao `id` do input.

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/use-unique-element-ids": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L54)
