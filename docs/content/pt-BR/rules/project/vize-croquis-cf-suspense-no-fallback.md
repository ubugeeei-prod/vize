---
title: "vize:croquis/cf/suspense-no-fallback"
---

# `vize:croquis/cf/suspense-no-fallback`

`<Suspense>` não tem conteúdo alternativo.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Suspense sem conteúdo alternativo é uma sintaxe válida do Vue. Esta é uma convenção escolhida de interface de carregamento, e não um erro do compilador; o contrato publicado não tem produtor atual.

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

O limite Suspense tem um filho assíncrono, mas não tem conteúdo alternativo, deixando este exemplo sem conteúdo de carregamento durante o estado pendente.

`App.vue`

```vue
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /></Suspense>
</template>

```

## Correto

O slot `#fallback` fornece um parágrafo explícito de carregamento até que o filho assíncrono seja resolvido.

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

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](../cross-file.md)
