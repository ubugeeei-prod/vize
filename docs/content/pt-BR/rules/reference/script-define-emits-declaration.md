---
title: "script/define-emits-declaration"
description: "Exigir a forma de defineEmits&lt;{}&gt;() baseada em tipos em vez da forma em tempo de execução ou de array."
---

# `script/define-emits-declaration`

Exigir a forma de defineEmits&lt;{}&gt;() baseada em tipos em vez da forma em tempo de execução ou de array.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`defineEmits(["change"])` usa uma declaração de array em tempo de execução; esta regra de estilo prefere uma declaração baseada em tipos.

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits(["change"]);
emit("change", 1);
</script>
```

## Correto

`defineEmits<{ change: [id: number] }>()` move a declaração de evento para um argumento de tipo e descreve explicitamente o dado numérico usado por `emit("change", 1)`.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits<{ change: [id: number] }>();
emit("change", 1);
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
        "script/define-emits-declaration": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39)
