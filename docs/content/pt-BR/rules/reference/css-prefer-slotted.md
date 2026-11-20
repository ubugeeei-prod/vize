---
title: "css/prefer-slotted"
description: "Recomendar ::v-slotted() para estilizar o conteúdo de slots."
---

# `css/prefer-slotted`

Recomendar ::v-slotted() para estilizar o conteúdo de slots.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Blocos style em CSS, SCSS, Sass, Less e Stylus, como escritos (sem pré-processador).
Em um mesmo bloco Stylus, use de forma consistente a sintaxe com chaves ou a sintaxe indentada; a sintaxe mista é lida como Stylus com chaves.

## Incorreto

A folha de estilos com escopo tem como alvo o ponto de inserção `slot`, em vez dos elementos fornecidos pelo slot.

```vue annotate="remove:2"
<style scoped>
slot { color: red; }
</style>
```

## Correto

`:slotted(.label)` tem como alvo o elemento de rótulo fornecido, por meio do seletor de slot com escopo.

```vue annotate="add:2"
<style scoped>
:slotted(.label) { color: red; }
</style>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/prefer-slotted": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras CSS](../css.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34)
