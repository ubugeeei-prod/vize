---
title: "vue/valid-model-definition"
description: "Exige chaves válidas na opção `model` do Vue 2."
---

# `vue/valid-model-definition`

Exige chaves válidas na opção `model` do Vue 2.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | Nenhuma | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

Esta regra opcional verifica a opção `model` do Vue 2. Apenas as chaves estáticas `prop` e `event` são aceitas; seus valores não são validados. O Vue 3 usa o contrato mais recente de `v-model` de componentes em vez dessa opção.

## Incorreto

`events` não é uma chave aceita na opção `model` de componentes do Vue 2.

```vue annotate="remove:2"
<script lang="ts">
export default { model: { prop: "value", events: "input" } };
</script>
```

## Correto

Use a chave aceita no singular, `event`, junto de `prop`, para descrever o contrato de model do Vue 2.

```vue annotate="add:2"
<script lang="ts">
export default { model: { prop: "value", event: "input" } };
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
        "vue/valid-model-definition": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_model_definition.rs#L21)
