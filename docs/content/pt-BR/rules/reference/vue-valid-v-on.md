---
title: "vue/valid-v-on"
description: "Exigir diretivas `v-on` válidas."
---

# `vue/valid-v-on`

Exigir diretivas `v-on` válidas.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Validação de modificadores

Modificadores não suportados são reportados individualmente no nome do modificador, inclusive na forma longa `v-on:click`, em argumentos de evento dinâmicos e na sintaxe de objeto de listeners. Os nomes aceitos correspondem aos do `eslint-plugin-vue` 10.9.2: modificadores de evento nativos do Vue, a tabela completa de aliases de teclas, qualquer ponto de código Unicode único e nomes com um prefixo inteiro decimal finito (conforme a semântica de `Number.parseInt(name, 10)` do JavaScript). Por exemplo, `.13suffix` é aceito por esta regra. Os diagnósticos de descontinuação de códigos numéricos de teclas e de migração de `.native` pertencem a regras separadas.

Apenas `.stop` e `.prevent` têm efeito sem um manipulador. Um modificador como `.once`, `.enter` ou um nome personalizado configurado ainda precisa de uma expressão de manipulador.

Projetos com tratamento personalizado de eventos podem adicionar nomes em `vue.linter.ruleOptions`:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/valid-v-on": "error" },
    ruleOptions: { "vue/valid-v-on": { modifiers: ["custom"] } },
  },
});
```

Isso permite `@click.custom="handler"`. Os nomes devem corresponder exatamente e diferenciam maiúsculas de minúsculas; as opções complementam os nomes nativos e não ativam uma regra desativada. Entradas de configuração com escopo específico podem substituir os nomes herdados por uma nova lista ou redefini-los com `[]`.

A tabela de teclas é adaptada dos [aliases de teclas do eslint-plugin-vue](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/key-aliases.json), licenciados sob a MIT. Seu código-fonte e sua licença são preservados junto à implementação.

## Incorreto

As formas de listener omitem um argumento de evento ou a expressão obrigatória de manipulador ou objeto.

```vue annotate="remove:2,3,4,5,6,7"
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
  <div @click.sync="handleClick"></div>
  <div @click.nativeish="handleClick"></div>
  <div @click.once></div>
</template>
```

## Correto

Use um evento com seu manipulador ou passe um objeto de listeners para `v-on` sem argumento.

```vue annotate="add:2,3,4,5,6"
<template>
  <div @click="handleClick"></div>
  <form @submit.prevent></form>
  <input @keyup.page-down="handlePageDown" />
  <input @keyup.あ="handleKey" />
  <div v-on="{ click: handleClick }"></div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-on": "error"
    },
    "ruleOptions": {
      "vue/valid-v-on": {
        "modifiers": [
          "custom"
        ]
      }
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30)
