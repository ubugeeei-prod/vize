---
title: "script/no-deprecated-dollar-listeners-api"
description: "Proibir a propriedade de instância $listeners removida no Vue 3 (incorporada a $attrs)."
---

# `script/no-deprecated-dollar-listeners-api`

Proibir a propriedade de instância $listeners removida no Vue 3 (incorporada a $attrs).

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

As leituras de membros e a referência direta no argumento usam `$listeners`, que o Vue 3 removeu após incorporar os ouvintes aos atributos.

```vue annotate="remove:4,5,9"
<script>
export default {
  mounted() {
    const vm = this
    return vm.$listeners
  }
}
</script>
<template><Child v-on="$listeners" /></template>
```

## Correto

As leituras passam a usar `this.$attrs` e `ctx.attrs` do contexto de setup. Essas formas substituem a API de ouvintes removida; os objetos receptores ilustrados precisam existir no contexto do componente ao redor.

```vue annotate="add:4,8"
<script>
export default {
  mounted() {
    return this.$attrs
  }
}
</script>
<template><Child v-bind="$attrs" /></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-dollar-listeners-api": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L54)
