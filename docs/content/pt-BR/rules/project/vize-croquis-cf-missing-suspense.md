---
title: "vize:croquis/cf/missing-suspense"
---

# `vize:croquis/cf/missing-suspense`

Uma dependência assíncrona é usada fora de um limite Suspense.

| Severidade padrão | Correção automática | Opções |
| --- | --- | --- |
| Não emitido | Nenhuma; revise os arquivos relacionados e aplique a correção | Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade |

Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

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

`AsyncCard.vue`

```vue
<script setup lang="ts">
const message = await Promise.resolve('Ready');
</script>

<template>
<p>{{ message }}</p>
</template>

```

## Incorreto

`AsyncCard` tem await no nível superior, tornando seu setup assíncrono, mas App o renderiza sem um limite Suspense para coordenar essa dependência.

`App.vue`

```vue
<script setup lang="ts">
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<AsyncCard />
</template>

```

## Correto

App envolve o filho assíncrono em `Suspense` e fornece conteúdo alternativo de carregamento até que o setup do filho seja resolvido.

`App.vue`

```vue
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /><template #fallback><p>Loading…</p></template></Suspense>
</template>

```

## Configuração (Vite+)

A CLI não emite este código atualmente, portanto não há uma configuração de regra da CLI para habilitá-lo aqui.

## Páginas relacionadas

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](../cross-file.md)
