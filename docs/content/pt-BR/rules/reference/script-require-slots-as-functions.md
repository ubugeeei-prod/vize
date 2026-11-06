---
title: "script/require-slots-as-functions"
description: "Exigir que as propriedades de $slots sejam chamadas como funções."
---

# `script/require-slots-as-functions`

Exigir que as propriedades de $slots sejam chamadas como funções.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`render` devolve `this.$slots.default` sem chamá-lo, que era o array de vnodes do Vue 2.

```vue annotate="remove:4"
<script lang="ts">
export default {
  render() {
    return this.$slots.default
  }
}
</script>
```

## Correto

`render` chama `this.$slots.default()` e usa o vnode devolvido.

```vue annotate="add:4"
<script lang="ts">
export default {
  render() {
    return this.$slots.default()
  }
}
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-slots-as-functions": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_slots_as_functions.rs#L44)
