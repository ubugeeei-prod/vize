---
title: "vue/no-reserved-component-names"
description: "Proibir o uso de nomes reservados como nomes de componentes."
---

# `vue/no-reserved-component-names`

Proibir o uso de nomes reservados como nomes de componentes.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O nome de componente button entra em conflito com o nome de um elemento HTML nativo.

```vue annotate="remove:1,2,3,4"
<script>
export default {
  name: "button",
};
</script>
```

## Correto

AppButton é um nome de componente da aplicação e não reutiliza o nome nativo button.

```vue annotate="add:1,2,4,5,6,7,8,9"
<script setup lang="ts">
defineOptions({ name: "AppButton" });
</script>

<template>
  <Transition>
    <AppButton />
  </Transition>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-reserved-component-names": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L44)
