---
title: "script/require-explicit-slots"
description: "Exigir que os slots consumidos por useSlots() sejam tipados explicitamente com defineSlots&lt;...&gt;()."
---

# `script/require-explicit-slots`

Exigir que os slots consumidos por useSlots() sejam tipados explicitamente com defineSlots&lt;...&gt;().

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

O `defineProps<{ id: number }>()` tipado estabelece a sintaxe TypeScript, mas setup usa `useSlots()` sem uma declaração de `defineSlots`. Assim, a regra encontra slots consumidos sem um contrato explícito de slots.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ id: number }>()
const slots = useSlots()
</script>
```

## Correto

`defineSlots` declara um slot `default` cujas props incluem `msg: string`; `useSlots()` agora aparece junto de um contrato de slots explicitamente tipado.

```vue annotate="add:2"
<script setup lang="ts">
defineSlots<{ default(props: { msg: string }): unknown }>()
const slots = useSlots()
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-explicit-slots": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92)
