---
title: "vize:croquis/cf/circular-dep"
---

# `vize:croquis/cf/circular-dep`

Os componentes importam uns aos outros em um ciclo.

| Severidade padrão | Correção automática | Opções |
| --- | --- | --- |
| Não emitido | Nenhuma; revise os arquivos relacionados e aplique a correção | Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade |

Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Isto ilustra um ciclo concreto de inicialização imediata. Um componente Vue recursivo ou qualquer importação circular não é automaticamente incorreto. Nenhum produtor atual emite este código de contrato.

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
import { aLabel } from './a';
</script>

<template>
<p>{{ aLabel }}</p>
</template>

```

`labels.ts`

```ts
export const aPrefix = 'A';
export const bPrefix = 'B';

```

## Incorreto

`a.ts` importa `b.ts`, que importa `a.ts` de volta. Ambos inicializam imediatamente uma constante a partir da constante ainda não inicializada do outro módulo, causando uma falha de zona morta temporal.

`a.ts`

```ts
import { bLabel } from './b';
export const aLabel = 'A' + bLabel;

```

`b.ts`

```ts
import { aLabel } from './a';
export const bLabel = 'B' + aLabel;

```

## Correto

Os dois módulos leem prefixos inicializados do módulo independente `labels.ts`, removendo o ciclo e a leitura cruzada durante a inicialização imediata.

`a.ts`

```ts
import { bPrefix } from './labels';
export const aLabel = 'A' + bPrefix;

```

`b.ts`

```ts
import { aPrefix } from './labels';
export const bLabel = 'B' + aPrefix;

```

## Configuração (Vite+)

A CLI não emite este código atualmente, portanto não há uma configuração de regra da CLI para habilitá-lo aqui.

## Páginas relacionadas

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](../cross-file.md)
