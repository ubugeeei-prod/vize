---
title: "script/no-side-effects-in-computed-properties"
description: "Proibir efeitos colaterais em getters computados da Options API."
---

# `script/no-side-effects-in-computed-properties`

Proibir efeitos colaterais em getters computados da Options API.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`doubled` atribui um valor a `this.count`, e `reversed` modifica `this.items` por meio de `reverse()`. Ambos os getters alteram o estado do qual deveriam derivar seus valores.

```vue annotate="remove:8,9,12"
<script lang="ts">
export default {
  data() {
    return { count: 0, items: [] }
  },
  computed: {
    doubled() {
      this.count = this.count * 2 // side effect: assigns to data
      return this.count
    },
    reversed() {
      return this.items.reverse() // side effect: mutates the array
    }
  }
}
</script>
```

## Correto

`doubled` retorna a multiplicação sem atribuição. `reversed` copia o array antes de invertê-lo, então o getter não altera o estado original do componente.

```vue annotate="add:8,11"
<script lang="ts">
export default {
  data() {
    return { count: 0, items: [] }
  },
  computed: {
    doubled() {
      return this.count * 2
    },
    reversed() {
      return [...this.items].reverse() // operate on a copy
    }
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
        "script/no-side-effects-in-computed-properties": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L77)
