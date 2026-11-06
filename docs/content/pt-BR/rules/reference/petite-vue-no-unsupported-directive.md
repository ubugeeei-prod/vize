---
title: "petite-vue/no-unsupported-directive"
description: "Proibir diretivas que o petite-vue não aceita."
---

# `petite-vue/no-unsupported-directive`

Proibir diretivas que o petite-vue não aceita.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | Nenhuma | Não | Nenhuma |

Aplicável a: Documentos HTML detectados como petite-vue; SFCs Vue comuns ficam fora do escopo desta regra.

## Incorreto

`v-memo`, `v-slot:header` e a diretiva personalizada `v-my-directive` não constam na lista de diretivas aceitas pelo petite-vue. O script do petite-vue identifica este HTML como o dialeto pertinente.

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-memo="[a, b]"></div>
<template v-slot:header></template>
<div v-my-directive></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## Correto

A substituição usa a sintaxe aceita de `v-scope`, `v-effect`, `v-if`, `v-bind` e `v-on`, em vez de depender de diretivas não aceitas.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "petite-vue/no-unsupported-directive": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de petite-vue](../petite-vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43)
