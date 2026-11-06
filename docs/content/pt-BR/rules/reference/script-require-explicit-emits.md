---
title: "script/require-explicit-emits"
description: "Exigir que os eventos emitidos sejam declarados em defineEmits ou na opção emits."
---

# `script/require-explicit-emits`

Exigir que os eventos emitidos sejam declarados em defineEmits ou na opção emits.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

A função de emissão capturada emite `save`, mas `defineEmits([])` não declara esse evento.

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits([]);
emit("save");
</script>
```

## Correto

Adicionar `"save"` à declaração torna o evento literal emitido parte do contrato explícito de eventos do componente.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(["save"]);
emit("save");
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-explicit-emits": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L60)
