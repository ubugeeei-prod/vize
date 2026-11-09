---
title: "a11y/click-events-have-key-events"
description: "Exigir manipuladores de eventos de teclado junto aos eventos de clique."
---

# `a11y/click-events-have-key-events`

Exigir manipuladores de eventos de teclado junto aos eventos de clique.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Verifica elementos não interativos sem uma função interativa. Botões nativos e elementos com uma função ARIA interativa ficam fora dos diagnósticos desta regra.

## Incorreto

A `div` não interativa tem um manipulador de clique, mas não trata eventos de teclado.

```vue annotate="remove:2"
<template>
<div @click="activate">Activate</div>
</template>
```

## Correto

Um `button` nativo permite a ativação pelo teclado para o mesmo manipulador `activate`.

```vue annotate="add:2"
<template>
<button @click="activate">Activate</button>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/click-events-have-key-events": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L25)
