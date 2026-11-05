---
title: "vize:croquis/cf/non-unique-id"
---

# `vize:croquis/cf/non-unique-id`

Um id de elemento dentro de um laço não é único por item.

| Severidade padrão | Correção automática | Opções |
| --- | --- | --- |
| error | Nenhuma; revise os arquivos relacionados e aplique a correção | Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade |

Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo

## Arquivos compartilhados do projeto

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ResultsList.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## Incorreto

Cada iteração de `v-for` renderiza o mesmo ID literal `result-title`; a chave do laço não torna os IDs do DOM únicos.

`ResultsList.vue`

```vue
<script setup lang="ts">
const results = [{ id: "first", title: "First result" }, { id: "second", title: "Second result" }];
</script>
<template>
  <article v-for="result in results" :key="result.id">
    <h2 id="result-title">{{ result.title }}</h2>
  </article>
</template>
```

## Correto

O ID do título inclui o ID estável do resultado, produzindo um identificador distinto no documento para cada item.

`ResultsList.vue`

```vue
<script setup lang="ts">
const results = [{ id: "first", title: "First result" }, { id: "second", title: "Second result" }];
</script>
<template>
  <article v-for="result in results" :key="result.id">
    <h2 :id="`result-${result.id}-title`">{{ result.title }}</h2>
  </article>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/non-unique-id": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## Páginas relacionadas

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/element_id.rs)

[Índice de verificações entre arquivos](../cross-file.md)
