---
title: "vize:croquis/cf/reactive-export"
---

# `vize:croquis/cf/reactive-export`

O estado reativo é exportado pelo módulo.

| Severidade padrão | Correção automática | Opções |
| --- | --- | --- |
| Não emitido | Nenhuma; revise os arquivos relacionados e aplique a correção | Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade |

Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Stores da aplicação compartilhados intencionalmente podem exportar estado reativo. Este cenário exige estado isolado e não afirma que toda exportação reativa seja inválida. Nenhum produtor atual emite este contrato.

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

## Incorreto

O módulo exporta um único objeto reativo inicializado, de modo que todo importador recebe a mesma contagem. Em um módulo SSR compartilhado entre requisições, isso impede o isolamento de estado por instância ou requisição pretendido no exemplo.

`state.ts`

```ts
import { reactive } from 'vue';
export const state = reactive({ count: 0 });

```

`App.vue`

```vue
<script setup lang="ts">
import { state } from './state';
</script>

<template>
<button @click="state.count++">{{ state.count }}</button>
</template>

```

## Correto

O módulo exporta uma fábrica, e App a chama dentro de setup. Cada instância obtém uma nova contagem reativa em vez da instância única exportada.

`state.ts`

```ts
import { reactive } from 'vue';
export function createState() { return reactive({ count: 0 }); }

```

`App.vue`

```vue
<script setup lang="ts">
import { createState } from './state';
const state = createState();
</script>

<template>
<button @click="state.count++">{{ state.count }}</button>
</template>

```

## Configuração (Vite+)

A CLI não emite este código atualmente, portanto não há uma configuração de regra da CLI para habilitá-lo aqui.

## Páginas relacionadas

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](../cross-file.md)
