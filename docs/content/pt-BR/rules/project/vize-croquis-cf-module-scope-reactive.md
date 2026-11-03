---
title: "vize:croquis/cf/module-scope-reactive"
---

# `vize:croquis/cf/module-scope-reactive`

O estado reativo é criado no escopo do módulo e compartilhado por todos os chamadores.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Estado reativo no escopo de módulo é válido para stores intencionais da aplicação. Este exemplo pressupõe isolamento por componente ou requisição; o contrato publicado atualmente não tem produtor.

## Arquivos compartilhados do projeto

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import Counter from './Counter.vue';
</script>

<template>
<Counter /><Counter />
</template>

```

`Counter.vue`

```vue
<script setup lang="ts">
import { createCounter } from './counter';
const { count } = createCounter();
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

## Incorreto

O módulo inicializa `count` uma vez, e as duas instâncias de Counter recebem a mesma ref. Clicar em uma altera os dois contadores, embora este exemplo pretenda manter um estado independente por instância.

`counter.ts`

```ts
import { ref } from 'vue';
const count = ref(0);
export function createCounter() { return { count }; }

```

## Correto

Criar a ref dentro de `createCounter` dá a cada chamada síncrona de setup um objeto de estado separado, de modo que cada botão possui seu contador.

`counter.ts`

```ts
import { ref } from 'vue';
export function createCounter() {
  const count = ref(0);
  return { count };
}

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](../cross-file.md)
