---
title: "css/no-important"
description: "Desencorajar o uso de !important no CSS."
---

# `css/no-important`

Desencorajar o uso de !important no CSS.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Blocos style em CSS puro (blocos scss, sass, less e stylus são ignorados).

## Incorreto

A declaração de cor substitui a prioridade normal da cascata com `!important`.

```vue annotate="remove:3"
<style scoped>
.button {
  color: red !important;
}
</style>
```

## Correto

A cor vem de uma propriedade personalizada sem uma declaração importante.

```vue annotate="add:3"
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "css/no-important": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras CSS](../css.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14)
