---
title: "script/no-unused-emit-declarations"
description: "Sinalizar eventos declarados que nunca são emitidos."
---

# `script/no-unused-emit-declarations`

Sinalizar eventos declarados que nunca são emitidos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`defineEmits` declara tanto `change` quanto `unused`, mas a função `emit` capturada só emite o evento literal `change`.

```vue annotate="remove:2,4"
<script setup lang="ts">
const emit = defineEmits(['change', 'unused'])
emit('change')
// `unused` is never emitted
</script>
```

## Correto

Remover `unused` faz a lista de eventos declarados corresponder à emissão observada. O exemplo usa uma variável emit capturada que não escapa do escopo, permitindo essa conclusão sobre o uso local.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(['change'])
emit('change')
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-unused-emit-declarations": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L82)
