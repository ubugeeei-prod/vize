---
title: "vue/no-non-component-keep-alive-child"
description: "Proibir invólucros de elementos comuns diretamente abaixo de `<KeepAlive>`."
---

# `vue/no-non-component-keep-alive-child`

Proibir invólucros de elementos comuns diretamente abaixo de `<KeepAlive>`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

KeepAlive envolve condicionalmente uma div nativa, em vez de armazenar UserCard diretamente em cache.

```vue annotate="remove:3"
<template>
  <KeepAlive>
    <div v-if="ready">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

## Correto

O primeiro exemplo torna UserCard o filho condicional. O invólucro com v-show ilustra uma estrutura fora desta verificação de filhos condicionais, sem prometer que o invólucro nativo seja armazenado em cache.

```vue annotate="add:3,4,5,6"
<template>
  <KeepAlive>
    <UserCard v-if="ready" />
  </KeepAlive>
  <KeepAlive>
    <div v-show="opened">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-non-component-keep-alive-child": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L26)
