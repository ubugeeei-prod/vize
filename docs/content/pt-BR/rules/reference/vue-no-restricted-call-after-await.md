---
title: "vue/no-restricted-call-after-await"
description: "Proíbe chamar um método restrito depois de await."
---

# `vue/no-restricted-call-after-await`

Proíbe chamar um método restrito depois de await.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O setup chama onMounted depois de await.

```vue annotate="remove:4,5"
<script lang="ts">
import { defineComponent, onMounted } from "vue";
export default defineComponent({
  async setup() {
    await Promise.resolve();
    onMounted(() => {});
  },
});
</script>
<template>
  <p>ok</p>
</template>
```

## Correto

O setup registra onMounted sem await.

```vue annotate="add:4"
<script lang="ts">
import { defineComponent, onMounted } from "vue";
export default defineComponent({
  setup() {
    onMounted(() => {});
  },
});
</script>
<template>
  <p>ok</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-call-after-await": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-call-after-await": [
        {
          "module": "vue",
          "path": "onMounted"
        }
      ]
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/await_call.rs#L25)
