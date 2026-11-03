---
title: "vize:croquis/cf/array-mutation"
---

# `vize:croquis/cf/array-mutation`

Um array é alterado por índice, o que um array reativo não rastreia.

| Severidade padrão | Correção automática | Opções |
| --- | --- | --- |
| Não emitido | Nenhuma; revise os arquivos relacionados e aplique a correção | Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade |

Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Apenas para o Vue 2.7 histórico: use dependências compatíveis do Vue 2.7 e do compilador de SFCs neste cenário. Os proxies do Vue 3 rastreiam atribuições a índices de arrays, então `items[0] = next` é reativo no Vue 3 e não é um defeito do Vue 3. Este código publicado não tem produtor atual.

## Arquivos compartilhados do projeto

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import Vue from 'vue';
import App from './App.vue';
new Vue({ render: h => h(App) }).$mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script lang="ts">
import Vue from 'vue';
import { replaceFirst } from './replace-first';
export default Vue.extend({
  data() { return { items: ['Before'] }; },
  methods: { replace() { replaceFirst(this.items, 'After'); } },
});
</script>
<template><section><p>{{ items[0] }}</p><button @click="replace">Replace</button></section></template>

```

## Incorreto

Neste projeto histórico com Vue 2.7, `items[0] = next` altera o array sem notificar o observador de arrays do Vue 2, de modo que o primeiro item exibido pode não ser atualizado.

`replace-first.ts`

```ts
export function replaceFirst(items: string[], next: string): void {
  items[0] = next;
}

```

## Correto

`splice(0, 1, next)` usa o método de mutação de arrays observado pelo Vue 2, permitindo que a mesma substituição atualize a visualização.

`replace-first.ts`

```ts
export function replaceFirst(items: string[], next: string): void {
  items.splice(0, 1, next);
}

```

## Configuração (Vite+)

A CLI não emite este código atualmente, portanto não há uma configuração de regra da CLI para habilitá-lo aqui.

## Páginas relacionadas

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](../cross-file.md)
