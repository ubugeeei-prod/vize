---
title: "a11y/no-access-key"
description: "Proibir o uso do atributo accesskey."
---

# `a11y/no-access-key`

Proibir o uso do atributo accesskey.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O atalho `accesskey="s"` pode entrar em conflito com atalhos do navegador ou de tecnologias assistivas.

```vue annotate="remove:2"
<template>
  <button accesskey="s">Save</button>
</template>
```

## Correto

Remover `accesskey` mantém disponível o botão Save comum.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "a11y/no-access-key": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L21)
