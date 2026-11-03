---
title: "vize:croquis/cf/multi-root-attrs"
---

# `vize:croquis/cf/multi-root-attrs`

Um componente com múltiplas raízes recebe atributos e não tem onde colocá-los.

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

O filho tem raízes `<main>` e `<aside>`, de modo que o Vue não tem uma raiz única que possa receber automaticamente a classe do pai.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main>Content</main><aside>Help</aside></template>
```

## Correto

Encaminhe `$attrs` explicitamente para `<main>`, mantendo a segunda raiz.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[Índice de verificações entre arquivos](../cross-file.md)
