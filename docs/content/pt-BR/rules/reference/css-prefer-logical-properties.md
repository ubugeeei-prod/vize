---
title: "css/prefer-logical-properties"
description: "Recomendar propriedades lógicas de CSS para melhorar o suporte à internacionalização."
---

# `css/prefer-logical-properties`

Recomendar propriedades lógicas de CSS para melhorar o suporte à internacionalização.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Blocos style em CSS puro (blocos scss, sass, less e stylus são ignorados).

## Incorreto

`margin-left` fixa a margem em um lado físico, independentemente da direção de escrita.

```vue annotate="remove:3"
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

## Correto

`margin-inline-start` acompanha o início da direção em linha.

```vue annotate="add:3"
<style scoped>
.panel {
  margin-inline-start: 1rem;
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
        "css/prefer-logical-properties": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de Musea e CSS](../musea-and-css.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15)
