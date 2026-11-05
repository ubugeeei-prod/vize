---
title: "vize:croquis/cf/setup-context-violation"
---

# `vize:croquis/cf/setup-context-violation`

Crie o estado reativo pertencente a cada componente no setup de sua instância.

| Severidade padrão | Correção automática | Opções |
| --- | --- | --- |
| dependente do contexto | Nenhuma; revise os arquivos relacionados e aplique a correção | Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade |

Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

## Escopo e limitações

Este cenário do analisador experimental coloca ref(0) no escopo do módulo de um script comum, em vez do setup por instância. O Vue permite esse estado reativo; o exemplo exige que cada instância do componente tenha sua própria contagem. A CLI não emite este código individual.

## Arquivos compartilhados do projeto

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## Incorreto

`ref(0)` é criada no escopo do módulo de um script comum, fora do contexto de setup por instância representado por este cenário do analisador.

`App.vue`

```vue
<script lang="ts">
import { ref } from "vue";
const count = ref(0);
export default {};
</script>
<template><p>Count</p></template>
```

## Correto

Mova a variável para script setup, onde cada instância de componente possui sua contagem e o template pode lê-la.

`App.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
</script>
<template><p>{{ count }}</p></template>
```

## Configuração (Vite+)

A CLI não emite este código atualmente, portanto não há uma configuração de regra da CLI para habilitá-lo aqui.

## Páginas relacionadas

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[Índice de verificações entre arquivos](../cross-file.md)
