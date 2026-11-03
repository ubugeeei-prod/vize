---
title: "script/order-in-components"
description: "Exigir a ordem padrão das propriedades nas opções do componente."
---

# `script/order-in-components`

Exigir a ordem padrão das propriedades nas opções do componente.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`data` está escrito antes de `props`, então um grupo posterior aparece acima de um grupo anterior.

```vue annotate="remove:5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 }
  },
  props: ['label']
}
</script>
```

## Correto

`props` vem antes de `data`, o que segue a ordem padrão dos grupos.

```vue annotate="add:3,6"
<script lang="ts">
export default {
  props: ['label'],
  data() {
    return { count: 0 }
  }
}
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/order-in-components": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/order_in_components.rs#L35)
