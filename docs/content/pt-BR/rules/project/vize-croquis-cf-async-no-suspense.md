---
title: "vize:croquis/cf/async-no-suspense"
---

# `vize:croquis/cf/async-no-suspense`

Um componente assíncrono é renderizado sem um limite Suspense.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

Suporte atual: `no-source-async-fact`

O produtor de diagnósticos de limites lê macros.is_async(), mas a análise do código-fonte atualmente registra o await de nível superior no escopo de script-setup. Portanto, o par completo de código-fonte incorreto e correto abaixo não produz um diagnóstico async-no-suspense pela CLI atual. Ele explica a convenção de Suspense; fornecer o fato ausente da macro é um trabalho posterior de implementação.

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

O filho tem await no nível superior, mas o pai não fornece um limite `<Suspense>`. A análise atual do código-fonte não fornece o fato de macro necessário para emitir este código de diagnóstico.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

## Correto

O pai envolve o mesmo filho assíncrono em `<Suspense>` com conteúdo alternativo de carregamento. Isso demonstra a convenção; a passagem atual continua sem emitir o diagnóstico para nenhuma das duas alternativas de código-fonte.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Suspense><Child /><template #fallback><p>Loading</p></template></Suspense></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[Índice de verificações entre arquivos](../cross-file.md)
