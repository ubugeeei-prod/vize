---
title: "css/prefer-nested-selectors"
description: "Recomendar o aninhamento de CSS para seletores de descendentes."
---

# `css/prefer-nested-selectors`

Recomendar o aninhamento de CSS para seletores de descendentes.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Blocos style em CSS puro (blocos scss, sass, less e stylus são ignorados).

## Incorreto

O seletor de descendentes `.card .title` repete o seletor pai em uma regra sem aninhamento.

```vue annotate="remove:2"
<style scoped>
.card .title { color: red; }
</style>
```

## Correto

A regra `.title` é aninhada dentro de `.card`, mantendo junta a relação de estilo entre pai e filho.

```vue annotate="add:2"
<style scoped>
.card { .title { color: red; } }
</style>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/prefer-nested-selectors": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras CSS](../css.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14)
