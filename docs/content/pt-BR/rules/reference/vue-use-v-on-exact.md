---
title: "vue/use-v-on-exact"
description: "Exigir o modificador `.exact` em `v-on` quando houver manipuladores baseados em modificadores."
---

# `vue/use-v-on-exact`

Exigir o modificador `.exact` em `v-on` quando houver manipuladores baseados em modificadores.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Quando os manipuladores se sobrepõem

A regra compara listeners do mesmo evento em um elemento nativo. Ela ignora eventos de componentes personalizados, a menos que ambos os listeners usem `.native`. Nomes de eventos diferentes e filtros de teclado sem teclas em comum não entram em conflito.

Os modificadores de sistema indicam teclas que precisam estar pressionadas, então `@keydown.alt.shift` também aceita Ctrl+Alt+Shift. Se um segundo manipulador usar `@keydown.alt.ctrl.shift`, adicione `.exact` ao manipulador mais abrangente para manter essas ações separadas. `.exact` permite os modificadores de sistema declarados e rejeita teclas Ctrl, Shift, Alt ou Meta adicionais.

Vários modificadores de teclado são alternativas: `@keydown.up.down` aceita qualquer uma das duas teclas de seta. Ele pode se sobrepor a `@keydown.ctrl.down` em Ctrl+SetaParaBaixo. A regra reporta a chave da diretiva que precisa de `.exact`, excluindo sua expressão de manipulador.

## Incorreto

O manipulador de clique comum também pode executar em Ctrl-clique, sobrepondo-se ao manipulador separado `.ctrl`.

```vue annotate="remove:2"
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

## Correto

`.exact` limita o manipulador de clique comum a cliques sem teclas modificadoras; o manipulador específico de Ctrl permanece separado.

```vue annotate="add:2,3,4,5,6"
<template>
  <button
    type="button"
    @click.exact="handleClick"
    @click.ctrl="handleCtrlClick"
  >
    Save
  </button>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/use-v-on-exact": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L31)
