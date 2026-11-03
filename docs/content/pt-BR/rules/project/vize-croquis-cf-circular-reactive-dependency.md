---
title: "vize:croquis/cf/circular-reactive-dependency"
---

# `vize:croquis/cf/circular-reactive-dependency`

Os cálculos reativos dependem uns dos outros em um ciclo.

Severidade padrão: dependente do contexto  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/circular-reactive-dependency": "warn" },
    },
  },
});
```

```sh
vp run lint
```

Qualificação do exemplo: `illustrative-source-pair`

O projeto Vue completo abaixo ilustra a realimentação de atualizações e sua correção. Ele não é uma comprovação qualificada de diagnóstico da CLI: o produtor de diagnósticos exige identidades de referências e arestas de fluxo reativo retidas, como mostra o grafo que acompanha o exemplo. Esses arquivos-fonte não demonstram que o processamento atual do código-fonte emitirá este código exato. Os controles específicos de diagnóstico de grafos com IDs rastreados permanecem separados das verificações de gramática do código-fonte.

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
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`count-key.ts`

```ts
import type { InjectionKey, Ref } from 'vue';
export const countKey: InjectionKey<Ref<number>> = Symbol('count');
```

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from 'vue';
import { countKey } from './count-key';
import CycleView from './CycleView.vue';
const count = ref(1); // A: the provider-owned source.
provide(countKey, count);
</script>
<template>
  <button @click="count++">Increment</button>
  <CycleView />
</template>
```

## Incorreto

App possui e fornece count (A). CycleView deriva nextCount (B) e imediatamente escreve cada valor derivado de volta no mesmo count injetado. Cada escrita altera novamente a entrada do cálculo, criando um ciclo de realimentação de atualizações A → B → A. As identidades no grafo preservado abaixo representam essas duas referências, não variáveis sem relação que tenham nomes iguais.

`CycleView.vue`

```vue
<script setup lang="ts">
import { computed, inject, watch } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
watch(nextCount, value => { count.value = value; }, { immediate: true });
</script>
<template><p>{{ nextCount }}</p></template>
```

```text
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B; B -> A
```

## Correto

Remova o observador que escreve B de volta em A. App mantém a propriedade de count e o altera apenas por meio de sua ação explícita Increment; CycleView lê o nextCount derivado sem realimentar o resultado. As mesmas referências preservam apenas a dependência A → B.

`CycleView.vue`

```vue
<script setup lang="ts">
import { computed, inject } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
</script>
<template><p>{{ nextCount }}</p></template>
```

```text
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Índice de verificações entre arquivos](../cross-file.md)
