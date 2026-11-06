---
title: "vue/scoped-event-names"
description: "Recomendar nomes de eventos com escopo no formato context:event."
---

# `vue/scoped-event-names`

Recomendar nomes de eventos com escopo no formato context:event.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

`playAudio`, `pauseAudio` e `reloadAudio` codificam seu escopo como sufixos camelCase, em vez de seguir a convenção de eventos separados por dois-pontos da regra.

```vue annotate="remove:3,4,5"
<template>
  <AudioPlayer
    @playAudio="play"
    @pauseAudio="pause"
    @reloadAudio="reload"
  />
</template>
```

## Correto

`audio:play`, `audio:pause` e `audio:reload` compartilham um escopo explícito `audio:`. O componente emissor deve usar os mesmos nomes.

```vue annotate="add:3,4,5"
<template>
  <AudioPlayer
    @audio:play="play"
    @audio:pause="pause"
    @audio:reload="reload"
  />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/scoped-event-names": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L31)
