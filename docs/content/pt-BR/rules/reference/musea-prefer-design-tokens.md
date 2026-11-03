---
title: "musea/prefer-design-tokens"
description: "Preferir variáveis CSS de tokens de design a valores primitivos fixos escritos diretamente."
---

# `musea/prefer-design-tokens`

Preferir variáveis CSS de tokens de design a valores primitivos fixos escritos diretamente.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Blocos art, variant e style de arquivos .art.vue do Musea.

Exige um arquivo .art.vue e o inventário de tokens mostrado abaixo. Não infere um token a partir de uma cor arbitrária.

## Incorreto

O exemplo art usa a cor azul literal, em vez do token de design primary configurado.

`Button.art.vue`

```vue annotate="remove:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

## Correto

O estilo faz referência a --color-primary, o token configurado para este exemplo.

`Button.art.vue`

```vue annotate="add:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: var(--color-primary);
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
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6",
              "tier": "semantic"
            }
          ]
        }
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de Musea e CSS](../musea-and-css.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32)
