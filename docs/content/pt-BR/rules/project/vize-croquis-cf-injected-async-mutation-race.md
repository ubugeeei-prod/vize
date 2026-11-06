---
title: "vize:croquis/cf/injected-async-mutation-race"
---

# `vize:croquis/cf/injected-async-mutation-race`

Um valor injetado é alterado por uma tarefa assíncrona sujeita a uma condição de corrida.

| Severidade padrão | Correção automática | Opções |
| --- | --- | --- |
| error | Nenhuma; revise os arquivos relacionados e aplique a correção | Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade |

Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo

## Arquivos compartilhados do projeto

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./StoreProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export async function loadCount(query: string, options?: { signal?: AbortSignal }): Promise<number> {
  const response = await fetch(`/count?q=${encodeURIComponent(query)}`, options);
  return Number(await response.text());
}
```

`CountSummary.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { StoreKey } from "./keys/store";
const store = inject(StoreKey)!;
</script>
<template><p>{{ store.count }}</p></template>
```

## Incorreto

`CountLoader.vue` escreve um resultado aguardado diretamente no armazenamento injetado compartilhado com `CountSummary.vue`, permitindo que um trabalho desatualizado afete os dois consumidores.

`keys/store.ts`

```ts
import type { InjectionKey } from "vue";

export interface Store {
  count: number;
}

export const StoreKey: InjectionKey<Store> = Symbol("store");
```

`StoreProvider.vue`

```vue
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);
</script>

<template>
  <CountLoader />
  <CountSummary />
</template>
```

`CountLoader.vue`

```vue
<script setup lang="ts">
import { loadCount } from "./api";
import { inject, ref, watch } from "vue";
import { StoreKey } from "./keys/store";

const store = inject(StoreKey)!;
const query = ref("");

watch(query, async (value) => {
  store.count = await loadCount(value);
});
</script>
```

## Correto

O carregador cancela trabalhos invalidados e emite apenas um resultado ativo. O provedor fica responsável pela alteração do armazenamento por meio de `applyLoadedCount`.

`keys/store.ts`

```ts
import type { InjectionKey } from "vue";

export interface Store {
  count: number;
}

export const StoreKey: InjectionKey<Store> = Symbol("store");
```

`StoreProvider.vue`

```vue
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);

function applyLoadedCount(count: number) {
  store.count = count;
}
</script>

<template>
  <CountLoader @loaded="applyLoadedCount" />
  <CountSummary />
</template>
```

`CountLoader.vue`

```vue
<script setup lang="ts">
import { loadCount } from "./api";
import { ref, watch } from "vue";

const emit = defineEmits<{ loaded: [count: number] }>();
const query = ref("");

watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;

  onCleanup(() => {
    active = false;
    controller.abort();
  });

  const count = await loadCount(value, { signal: controller.signal });
  if (active) emit("loaded", count);
});
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "incremental", crossFile: true,
    rules: { "croquis/cf/injected-async-mutation-race": "warn" },
  },
});
```

```sh
vp run lint
```

## Páginas relacionadas

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[Índice de verificações entre arquivos](../cross-file.md)
