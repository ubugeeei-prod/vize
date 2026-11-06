---
title: "a11y/img-alt"
description: "Exigir o atributo alt nas imagens para garantir acessibilidade."
---

# `a11y/img-alt`

Exigir o atributo alt nas imagens para garantir acessibilidade.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A imagem do avatar não tem o atributo `alt`.

```vue annotate="remove:2"
<template>
  <img src="/avatar.png" />
</template>
```

## Correto

`alt="User avatar"` fornece uma alternativa textual para o avatar.

```vue annotate="add:2"
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/img-alt": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L17)
