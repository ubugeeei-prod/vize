---
title: "a11y/role-img-has-name"
description: "Exigir um nome acessível nos elementos com role=\"img\"."
---

# `a11y/role-img-has-name`

Exigir um nome acessível nos elementos com role="img".

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A imagem define `role="img"` depois de um `alt` vazio, então fica exposta como uma imagem sem nome acessível.

```vue annotate="remove:2"
<template>
  <img src="/a.png" alt="" role="img" />
</template>
```

## Correto

`alt="Network diagram"` nomeia a imagem, então `role="img"` tem um nome acessível.

```vue annotate="add:2"
<template>
  <img src="/a.png" alt="Network diagram" role="img" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/role-img-has-name": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_img_has_name.rs#L14)
