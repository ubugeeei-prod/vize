---
title: "script/no-options-api"
description: "Proibir padrões da Options API no modo Vapor."
---

# `script/no-options-api`

Proibir padrões da Options API no modo Vapor.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `opinionated` | Não | Nenhuma |

Aplicável a: Verificações de script voltadas ao Vapor; a ativação explícita também aplica a restrição a scripts comuns.

## Incorreto

O objeto da exportação padrão declara `data()` da Options API, uma forma de opção de componente proibida por esta regra.

```vue annotate="remove:1,2,3,4,5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

## Correto

O estado do componente passa a ser um `ref` da Composition API em `<script setup>` com Vapor, removendo o objeto da Options API e sua opção `data`.

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-options-api": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43)
