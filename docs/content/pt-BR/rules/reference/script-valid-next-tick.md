---
title: "script/valid-next-tick"
description: "Exigir que o resultado de uma chamada de nextTick() seja aguardado, encadeado ou receba uma função de retorno."
---

# `script/valid-next-tick`

Exigir que o resultado de uma chamada de nextTick() seja aguardado, encadeado ou receba uma função de retorno.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

O `nextTick()` importado é uma expressão isolada sem função de retorno, então a Promise retornada é ignorada e nenhum trabalho aguarda a atualização do DOM.

```vue annotate="remove:3"
<script setup lang="ts">
import { nextTick } from "vue";
nextTick();
</script>
```

## Correto

`await nextTick()` consome a Promise e aguarda explicitamente a próxima atualização do DOM antes que o código subsequente de setup continue.

```vue annotate="add:3"
<script setup lang="ts">
import { nextTick } from "vue";
await nextTick();
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
        "script/valid-next-tick": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55)
