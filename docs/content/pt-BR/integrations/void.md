---
title: Void
description: Use o Vize em um app Void com páginas @void/vue, SSR no workerd e os tipos gerados pelo Void.
---

<!-- Generated translation; source: integrations/void.md -->

# Void

O [Void](https://void.cloud) cria apps full-stack sobre o Vite e os executa no workerd da Cloudflare. Seu adaptador Vue, `@void/vue`, renderiza `pages/` no servidor e faz a hidratação no navegador. Adicione `vize()` ao lado dele: o Vize compila cada arquivo `.vue`, e o Void mantém o roteamento, os loaders, as ilhas e os bindings.

> [!WARNING]
> O Vize está em desenvolvimento ativo. Compare a saída do build e os diagnósticos no seu próprio app antes de adotá-lo e consulte o [status de suporte](../stability.md).

## Configuração

### 1. Instale o plugin

```bash
npm install -D @vizejs/vite-plugin
```

### 2. Adicione `vize()` ao `vite.config.ts`

```ts
import { voidVue } from "@void/vue/plugin";
import { vize } from "@vizejs/vite-plugin";
import { defineConfig } from "vite";
import { voidPlugin } from "void";

export default defineConfig({
  plugins: [voidPlugin(), vize(), voidVue()],
});
```

`voidVue()` inclui seu próprio `@vitejs/plugin-vue`. Não é preciso removê-lo: `vize()` o substitui para que um único compilador cuide de cada SFC. Se você definir `exclude`, o plugin-vue permanece e compila os arquivos que o Vize ignora.

### 3. Execute o Void normalmente

```bash
npx vite dev
npx vite build
```

Nenhuma outra mudança é necessária:

- **Renderização no servidor com workerd.** Vue e `vue/server-renderer` são empacotados no worker, já que o workerd não tem `node_modules` para importar.
- **CSS e preloads das rotas.** O manifesto do cliente usa o caminho do código-fonte (`pages/index.vue`) como chave, a mesma que o Void usa para incluir a folha de estilo da rota no HTML renderizado no servidor.
- **HMR no `vite dev`.** Editar um template mantém o estado do componente. As props dos loaders continuam no lugar.
- **Props de página.** `defineProps<Props>()` com `Props` importado de um loader `.server.ts` compila como no plugin-vue.

## Verificação de tipos

O Void gera `.void/tsconfig.json` com os tipos de rotas, ambiente e bindings; o seu `tsconfig.json` o estende. Rode `void prepare` uma vez (ou inicie o servidor de desenvolvimento) para criar esses arquivos e depois verifique:

```bash
npx void prepare
vize check
```

O `vize check` lê os `files` e `paths` gerados, então nos arquivos `.vue` estes itens têm os mesmos tipos que no `vue-tsc`:

- props dos loaders via `InferProps<typeof loader>`;
- `fetch` de `void/client` (caminhos, métodos e corpos de resposta das rotas);
- `env` de `void/env` e módulos de binding como `void/kv`.

## Lint e formatação

`vize lint` e `vize fmt` encontram seus arquivos a partir da raiz do projeto e ignoram o que está no `.gitignore`. O `.gitignore` do Void cobre `dist`, `.void`, `.wrangler` e `.void-wrangler.jsonc`, então arquivos gerados não são analisados nem reformatados.

## Editor

O [servidor de linguagem](../guide/language-server.md) usa o mesmo `tsconfig.json`. Rode `void prepare` antes de abrir o projeto para que os tipos gerados existam.

## Como é testado

A cada mudança no plugin Vite ou no compilador nativo, a CI constrói um app Void 1.1.12 fixado com `@void/vue` (`tools/support/compat/void/fixtures/void-vue`) e verifica:

- o build de produção e o SSR no `vite preview` com workerd (incluindo o CSS das rotas e um binding KV);
- SSR em desenvolvimento, hidratação e HMR que preserva o estado;
- que `vize check` e `vue-tsc` relatam os mesmos erros;
- `vize lint` e `vize fmt --check`.

Nuxt no Void (`voidPlugin()` no `nuxt.config.ts`) ainda não é coberto por esta fixture. Nesse caso, use a [integração Nuxt](./nuxt.md).
