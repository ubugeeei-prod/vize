---
title: "script/no-multiple-slot-args"
description: "Proibir passar mais de um argumento a uma chamada de função de slot com escopo."
---

# `script/no-multiple-slot-args`

Proibir passar mais de um argumento a uma chamada de função de slot com escopo.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

As chamadas de slot passam vários argumentos posicionais ou expandem uma lista desconhecida de argumentos. Os slots do Vue recebem um único objeto de props, não uma lista de parâmetros posicionais.

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
slots.default(foo, bar)
$slots.header(a, b)
this.$scopedSlots.item(x, y)
useSlots().default(a, b)
slots.default(...args)
</script>
```

## Correto

`{ foo, bar }` combina os dados em um único argumento; `slotProps` e a chamada sem argumentos também respeitam a forma de chamada de slot suportada.

```vue annotate="add:2,3,4"
<script setup lang="ts">
slots.default({ foo, bar })
slots.default(slotProps)
slots.default()
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-multiple-slot-args": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61)
