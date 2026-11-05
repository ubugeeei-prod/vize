---
title: "vize:croquis/cf/object-identity-comparison"
---

# `vize:croquis/cf/object-identity-comparison`

Compare IDs estáveis de registros entre objetos brutos e proxies reativos.

| Severidade padrão | Correção automática | Opções |
| --- | --- | --- |
| Não emitido | Nenhuma; revise os arquivos relacionados e aplique a correção | Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade |

Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

O exemplo pressupõe que os IDs identifiquem os registros de forma única. Comparar duas referências ao mesmo proxy reativo continua válido; este contrato não tem produtor atual. Um objeto bruto e seu proxy reativo têm identidades distintas, e toRaw() devolve o objeto original.

## Escopo e limitações

Um objeto bruto e seu proxy reativo têm identidades de objeto distintas, e toRaw() retorna o objeto original. Este contrato ilustra a comparação de IDs estáveis de registros quando se pretende comparar a identidade do registro.

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

`user.ts`

```ts
import { reactive } from 'vue';
export function makeUser() {
  const raw = { id: 7, name: 'Ada' };
  return { raw, proxy: reactive(raw) };
}

```

## Incorreto

Este contrato ilustra a comparação de IDs estáveis quando a identidade pretendida é a do registro. `proxy === raw` compara a identidade do invólucro, portanto é falso mesmo que ambos representem o mesmo registro de usuário. A aplicação pretendia comparar a identidade do registro, não a do invólucro do objeto.

`App.vue`

```vue
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy === raw;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

## Correto

Comparar o `id` estável do registro responde à pergunta pretendida sem depender de o objeto ser bruto ou estar envolvido por um proxy.

`App.vue`

```vue
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy.id === raw.id;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

## Configuração (Vite+)

A CLI não emite este código atualmente, portanto não há uma configuração de regra da CLI para habilitá-lo aqui.

## Páginas relacionadas

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](../cross-file.md)
