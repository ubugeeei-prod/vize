---
title: "script/component-options-name-casing"
description: "Exigir PascalCase na opção `name` do componente."
---

# `script/component-options-name-casing`

Exigir PascalCase na opção `name` do componente.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

A opção de componente `name: 'my-component'` está em kebab-case, enquanto esta regra exige um nome de componente literal em PascalCase.

```vue annotate="remove:3"
<script lang="ts">
export default {
  name: 'my-component' // kebab-case
}
</script>
```

## Correto

`MyComponent` começa com uma letra maiúscula e contém apenas caracteres alfanuméricos, atendendo à verificação de nome.

```vue annotate="add:3"
<script lang="ts">
export default {
  name: 'MyComponent'
}
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
        "script/component-options-name-casing": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44)
