---
title: "script/no-computed-properties-in-data"
description: "Proibir o uso de propriedades computadas dentro da função data do componente."
---

# `script/no-computed-properties-in-data`

Proibir o uso de propriedades computadas dentro da função data do componente.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`data()` devolve `this.doubled` enquanto `doubled` ainda é uma propriedade computada, lida antes da inicialização.

```vue annotate="remove:4,7"
<script lang="ts">
export default {
  data() {
    return { value: this.doubled }
  },
  computed: {
    doubled() { return 1 }
  }
}
</script>
```

## Correto

`data()` devolve um valor simples, e `doubled` lê `this.value` depois da inicialização.

```vue annotate="add:4,7"
<script lang="ts">
export default {
  data() {
    return { value: 1 }
  },
  computed: {
    doubled() { return this.value * 2 }
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
        "script/no-computed-properties-in-data": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_computed_properties_in_data.rs#L44)
