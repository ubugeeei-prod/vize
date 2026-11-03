---
title: "script/require-typed-ref"
description: "Exigir um argumento de tipo explícito em um ref() inicializado sem valor, com null ou com undefined."
---

# `script/require-typed-ref`

Exigir um argumento de tipo explícito em um ref() inicializado sem valor, com null ou com undefined.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

As chamadas do `ref` importado não têm um argumento de tipo nem um valor inicial útil: a ausência de argumento, `null` e `undefined` não permitem inferir o tipo pretendido para o valor futuro.

```vue annotate="remove:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref()           // Ref<undefined>
const b = ref(null)       // Ref<null>
const c = ref(undefined)  // Ref<undefined>
</script>
```

## Correto

Os argumentos de tipo explícitos descrevem os refs de string e de User que aceita null. `ref(0)` já tem um inicializador numérico concreto e pode depender da inferência.

```vue annotate="add:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref<string>()
const b = ref<User | null>(null)
const c = ref(0)          // inferred Ref<number>
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
        "script/require-typed-ref": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55)
