---
title: "script/consistent-props-emits-key-quotes"
description: "Uniformizar chaves entre aspas nas declarações de props e emits."
---

# `script/consistent-props-emits-key-quotes`

Uniformizar chaves entre aspas nas declarações de props e emits.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

A declaração de emits mistura a chave `update:modelValue`, que precisa de aspas, com a chave `close` sem aspas.

```vue annotate="remove:4"
<script setup lang="ts">
const emit = defineEmits<{
  "update:modelValue": [value: string]
  close: []
}>()
</script>
```

## Correto

Todas as chaves estáticas da declaração usam aspas, pois `update:modelValue` não pode ser escrito como um identificador JavaScript sem aspas.

```vue annotate="add:4,6,7,8,9"
<script setup lang="ts">
const emit = defineEmits<{
  "update:modelValue": [value: string]
  "close": []
}>()
const props = defineProps({
  "aria-label": String,
  "disabled": Boolean,
})
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/consistent-props-emits-key-quotes": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/consistent_key_quotes.rs#L49)
