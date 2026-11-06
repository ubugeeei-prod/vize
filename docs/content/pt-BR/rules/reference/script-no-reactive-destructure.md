---
title: "script/no-reactive-destructure"
description: "Proibir a desestruturação de objetos reativos que causa perda de reatividade."
---

# `script/no-reactive-destructure`

Proibir a desestruturação de objetos reativos que causa perda de reatividade.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`const { count, name } = state` copia propriedades primitivas do objeto `reactive`, perdendo sua conexão com alterações posteriores nas propriedades.

```vue annotate="remove:2,4"
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = state;
</script>
```

## Correto

Desestruturar `toRefs(state)` cria refs para `count` e `name`, mantendo cada variável vinculada à propriedade reativa original.

```vue annotate="add:2,4"
<script setup lang="ts">
import { reactive, toRefs } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = toRefs(state);
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-reactive-destructure": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43)
