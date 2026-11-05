---
title: "script/define-macros-order"
description: "Exigir uma ordem consistente para as macros do compilador Vue em &lt;script setup&gt;."
---

# `script/define-macros-order`

Exigir uma ordem consistente para as macros do compilador Vue em &lt;script setup&gt;.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`defineEmits` aparece antes de `defineProps`, mas `defineProps` e depois `defineEmits` devem ser as primeiras instruções de `<script setup>`.

```vue annotate="remove:2,3"
<script setup lang="ts">
// defineEmits before defineProps
const emit = defineEmits<{ change: [value: string] }>()
const props = defineProps<{ count: number }>()
</script>
```

## Correto

`defineProps` e depois `defineEmits` vêm logo após os imports, antes de qualquer outra instrução; as demais macros podem vir em qualquer ordem.

```vue annotate="add:2,4,5,6"
<script setup lang="ts">
import { ref } from 'vue'
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
const model = defineModel<string>()
const value = ref(0)
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
        "script/define-macros-order": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L44)
