---
title: "vue/no-mutating-props"
description: "Proibir a mutação de props de componentes."
---

# `vue/no-mutating-props`

Proibir a mutação de props de componentes.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Uma asserção de tipo não altera o valor que envolve; por isso, em um template, `(rows as string[]).push(x)` e `rows!.push(x)` são relatados como mutações da prop `rows`, enquanto o eslint-plugin-vue para na asserção.

## Incorreto

Incrementar props.count escreve diretamente em um valor fornecido pelo componente pai.

```vue annotate="remove:4"
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

## Correto

O componente emite update:count com o próximo valor, deixando o pai responsável por atualizar a prop.

```vue annotate="add:3,5,6,7"
<script setup lang="ts">
const props = defineProps<{ count: number }>();
const emit = defineEmits<{ "update:count": [value: number] }>();

function increment() {
  emit("update:count", props.count + 1);
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
      "vue/no-mutating-props": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L36)
