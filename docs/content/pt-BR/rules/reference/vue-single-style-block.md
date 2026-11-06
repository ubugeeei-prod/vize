---
title: "vue/single-style-block"
description: "Recomendar um único bloco style."
---

# `vue/single-style-block`

Recomendar um único bloco style.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O componente divide seus estilos com escopo de panel e title entre dois blocos style.

```vue annotate="remove:5,6,7"
<style scoped>
.panel {
  color: red;
}
</style>

<style scoped>
.title {
  color: blue;
}
</style>
```

## Correto

Os dois seletores mantêm o escopo em um único bloco style, atendendo à convenção de bloco único sem remover nenhum estilo.

```vue
<style scoped>
.panel {
  color: red;
}
.title {
  color: blue;
}
</style>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/single-style-block": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41)
