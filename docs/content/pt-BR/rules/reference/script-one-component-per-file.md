---
title: "script/one-component-per-file"
description: "Proibir mais de uma definição de componente em um arquivo."
---

# `script/one-component-per-file`

Proibir mais de uma definição de componente em um arquivo.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

O arquivo exporta `Parent` e também define um componente `Child` em `components`.

```vue annotate="remove:1,2,3"
<script setup lang="ts">
const First = defineComponent({ name: 'First' })
const Second = defineComponent({ name: 'Second' })
</script>
```

## Correto

O arquivo define apenas `Parent`.

```vue annotate="add:1,2,3,4"
<script lang="ts">
export default {
  name: 'Parent'
}
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/one-component-per-file": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/one_component_per_file.rs#L26)
