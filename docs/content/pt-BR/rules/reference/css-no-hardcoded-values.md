---
title: "css/no-hardcoded-values"
description: "Sugerir variáveis CSS em vez de valores fixos escritos diretamente."
---

# `css/no-hardcoded-values`

Sugerir variáveis CSS em vez de valores fixos escritos diretamente.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Blocos style em CSS puro (blocos scss, sass, less e stylus são ignorados).

## Incorreto

O botão incorpora números de espaçamento e uma cor hexadecimal diretamente nas declarações.

```vue annotate="remove:3,4"
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

## Correto

As declarações fazem referência a propriedades personalizadas de espaçamento e cor com nomes, permitindo manter esses valores como tokens.

```vue annotate="add:3,4"
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
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
        "css/no-hardcoded-values": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras CSS](../css.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_hardcoded_values.rs#L30)
