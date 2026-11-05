---
title: "a11y/no-distracting-elements"
description: "Proibir elementos que causam distração, como &lt;marquee&gt; e &lt;blink&gt;."
---

# `a11y/no-distracting-elements`

Proibir elementos que causam distração, como &lt;marquee&gt; e &lt;blink&gt;.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O elemento `marquee` introduz texto que se move automaticamente.

```vue annotate="remove:2"
<template>
  <marquee>Limited offer</marquee>
</template>
```

## Correto

Um parágrafo exibe a mesma oferta sem o elemento marquee que causa distração.

```vue annotate="add:2"
<template>
  <p>Limited offer</p>
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
        "a11y/no-distracting-elements": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16)
