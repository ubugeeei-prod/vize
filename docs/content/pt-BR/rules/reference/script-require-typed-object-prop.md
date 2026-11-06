---
title: "script/require-typed-object-prop"
description: "Exigir um tipo explícito em uma prop cujo tipo de tempo de execução seja `Object` ou `Array`."
---

# `script/require-typed-object-prop`

Exigir um tipo explícito em uma prop cujo tipo de tempo de execução seja `Object` ou `Array`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

Os construtores simples `Object` e `Array` descrevem apenas categorias amplas de tempo de execução, então nem `user` nem a estrutura dos elementos de `items` têm um tipo estático explícito.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ user: Object, items: { type: Array } });
</script>
```

## Correto

`PropType<User>` e `PropType<User[]>` adicionam os tipos de objeto e de elemento enquanto mantêm os mesmos construtores de tempo de execução.

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
import type { PropType } from "vue";
interface User { name: string }
const props = defineProps({
  user: Object as PropType<User>,
  items: { type: Array as PropType<User[]> },
});
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-typed-object-prop": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63)
