---
title: "vue/prefer-prop-type-boolean-first"
description: "Exige `Boolean` na primeira posição da lista de construtores de tipo das props."
---

# `vue/prefer-prop-type-boolean-first`

Exige `Boolean` na primeira posição da lista de construtores de tipo das props.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

Alterar a ordem dos construtores pode mudar a conversão booleana em tempo de execução. Esta verificação opcional abrange o `defineProps` em tempo de execução e as listas de tipos de props da Options API; ela não reordena uniões de tipos do TypeScript.

## Incorreto

`String` vem antes de `Boolean`, então um atributo `enabled` sem valor usa a conversão que prioriza strings em vez da convenção que prioriza booleanos.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps({ enabled: [String, Boolean] });
</script>
```

## Correto

Coloque `Boolean` primeiro para priorizar a conversão booleana da prop, mantendo a aceitação do tipo string.

```vue annotate="add:2"
<script setup lang="ts">
defineProps({ enabled: [Boolean, String] });
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-prop-type-boolean-first": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/prefer_prop_type_boolean_first.rs#L19)
