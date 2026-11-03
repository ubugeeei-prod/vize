---
title: "vize:croquis/cf/unused-emit"
---

# `vize:croquis/cf/unused-emit`

Um evento declarado nunca é emitido.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

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

O filho declara `save`, mas nunca chama a função de emissão de eventos com esse nome.

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
const emit = defineEmits<{ save: [] }>();
</script>
<template><p>Content</p></template>
```

## Correto

O exemplo chama `emit("save")`, fazendo com que o evento declarado seja usado. Interações reais devem emiti-lo quando a ação correspondente ocorrer.

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
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[Índice de verificações entre arquivos](../cross-file.md)
