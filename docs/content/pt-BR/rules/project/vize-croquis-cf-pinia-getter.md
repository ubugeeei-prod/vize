---
title: "vize:croquis/cf/pinia-getter"
---

# `vize:croquis/cf/pinia-getter`

Evite copiar valores de getters do Pinia como instantâneos durante setup.

| Severidade padrão | Correção automática | Opções |
| --- | --- | --- |
| Não emitido | Nenhuma; revise os arquivos relacionados e aplique a correção | Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade |

Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Pinia deve estar instalado, e main.ts instala seu plugin antes da montagem. Ler `store.doubled` diretamente dentro de um cálculo rastreado ou de um template é válido; o defeito aqui é capturar um valor instantâneo comum. Este contrato atualmente não tem produtor.

## Escopo e limitações

Este contrato ilustra a cópia do valor primitivo atual de um getter do Pinia durante setup, criando um instantâneo que deixa de acompanhar atualizações. A leitura direta do getter em um cálculo rastreado ou no template continua reativa; storeToRefs() é a forma de extração usada aqui para corrigir o problema.

## Arquivos compartilhados do projeto

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
createApp(App).use(createPinia()).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`counter-store.ts`

```ts
import { defineStore } from 'pinia';
export const useCounterStore = defineStore('counter', {
  state: () => ({ count: 0 }),
  getters: { doubled: state => state.count * 2 },
});

```

## Incorreto

`const doubled = store.doubled` copia o número atual da função de leitura durante setup. O número copiado não acompanha atualizações posteriores de `store.count`.

`App.vue`

```vue
<script setup lang="ts">
import { useCounterStore } from './counter-store';
const store = useCounterStore();
const doubled = store.doubled;
</script>

<template>
<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>
</template>

```

## Correto

`storeToRefs(store)` fornece uma ref reativa da função de leitura que pode ser desestruturada e desembrulhada pelo template, mantendo a conexão com o armazenamento.

`App.vue`

```vue
<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useCounterStore } from './counter-store';
const store = useCounterStore();
const { doubled } = storeToRefs(store);
</script>

<template>
<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>
</template>

```

## Configuração (Vite+)

A CLI não emite este código atualmente, portanto não há uma configuração de regra da CLI para habilitá-lo aqui.

## Páginas relacionadas

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](../cross-file.md)
