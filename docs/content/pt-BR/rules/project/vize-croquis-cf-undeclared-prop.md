---
title: "vize:croquis/cf/undeclared-prop"
---

# `vize:croquis/cf/undeclared-prop`

Um pai passa uma prop que o filho não declara.

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

O pai passa `typo`, embora o filho resolvido declare apenas `title`. Essa convenção do analisador é distinta do comportamento geral de herança automática de atributos do Vue.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" :typo="true" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

## Correto

Remova a vinculação não intencional de `typo` e mantenha a prop declarada `title`.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[Índice de verificações entre arquivos](../cross-file.md)
