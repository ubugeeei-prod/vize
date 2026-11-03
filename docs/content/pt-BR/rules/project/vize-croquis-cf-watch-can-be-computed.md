---
title: "vize:croquis/cf/watch-can-be-computed"
---

# `vize:croquis/cf/watch-can-be-computed`

Um observador apenas copia um valor para o estado e pode ser um valor computado.

| Severidade padrão | Correção automática | Opções |
| --- | --- | --- |
| Não emitido | Nenhuma; revise os arquivos relacionados e aplique a correção | Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade |

Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Isto ilustra a preferência publicada por estado puramente derivado. Watchers continuam adequados para efeitos externos ou estado gravável de forma independente; nenhum produtor atual emite este contrato.

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
import { useDouble } from './use-double';
const { count, doubled } = useDouble();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ doubled }}</p>
</template>

```

## Incorreto

O observador não produz nenhum efeito externo; ele apenas mantém uma segunda ref gravável sincronizada com o dobro de `count`. Este exemplo não tem escritas independentes nesse valor derivado.

`use-double.ts`

```ts
import { ref, watch } from 'vue';
export function useDouble() {
  const count = ref(0);
  const doubled = ref(0);
  watch(count, next => { doubled.value = next * 2; }, { immediate: true });
  return { count, doubled };
}

```

## Correto

Uma função de leitura computada expressa diretamente a mesma derivação e remove a sincronização manual e o estado gravável adicional.

`use-double.ts`

```ts
import { computed, ref } from 'vue';
export function useDouble() {
  const count = ref(0);
  const doubled = computed(() => count.value * 2);
  return { count, doubled };
}

```

## Configuração (Vite+)

A CLI não emite este código atualmente, portanto não há uma configuração de regra da CLI para habilitá-lo aqui.

## Páginas relacionadas

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](../cross-file.md)
