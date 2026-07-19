---
title: Integrações experimentais de bundlers
---

<!-- Generated translation; source: guide/unplugin.md -->

# Integrações experimentais de bundlers

> **⚠️ Experimental:** `@vizejs/unplugin` e `@vizejs/rspack-plugin` ainda são instáveis.
> `@vizejs/vite-plugin` continua sendo a integração de bundlers recomendada e mais testada até hoje.

Vize oferece um pacote experimental de [unplugin](https://unplugin.unjs.io/) para `rollup`, `webpack`e `esbuild`, além de um pacote dedicado `Rspack` :

- `@vizejs/unplugin` — `rollup` / `webpack` / `esbuild`
- `@vizejs/rspack-plugin` — `Rspack` apenas

O RSPACK **intencionalmente não** passa pelo caminho compartilhado de desplugin.
Sua cadeia de carga, `experiments.css`e comportamento do HMR precisam de manuseio específico do Rspack.

## Instalação

Instale `vp` uma vez a partir do [Vite+ install guide](https://viteplus.dev/guide/install), depois adicione os pacotes:

```bash
vp install @vizejs/unplugin
```

Para o Rspack:

```bash
vp install -D @vizejs/rspack-plugin @rspack/core
```

## Rolagem

```javascript
// rollup.config.mjs
import vize from "@vizejs/unplugin/rollup";

export default {
  plugins: [vize()],
};
```

## webpack

```javascript
// webpack.config.mjs
import Vize from "@vizejs/unplugin/webpack";

export default {
  plugins: [Vize()],
};
```

## ESBUILD

```javascript
// build.mjs
import { build } from "esbuild";
import vize from "@vizejs/unplugin/esbuild";

await build({
  entryPoints: ["src/main.ts"],
  bundle: true,
  plugins: [vize()],
});
```

## Rspack

Use o pacote dedicado `@vizejs/rspack-plugin` em vez de `@vizejs/unplugin`:

```javascript
// rspack.config.mjs
import { VizePlugin } from "@vizejs/rspack-plugin";

export default {
  experiments: {
    css: true,
  },
  module: {
    rules: [
      {
        test: /\.vue$/,
        loader: "@vizejs/rspack-plugin/loader",
      },
    ],
  },
  plugins: [new VizePlugin()],
};
```

Veja o pacote README para a superfície completa da configuração do Rspack.

## Ressalvas

- O Vite ainda é a integração recomendada se você precisa do comportamento mais completo e testado.
- Módulos CSS e pré-processadores de estilo fora do Vite dependem do pipeline CSS do bundler host e têm mais probabilidade de mudar.
- Se seu bundler inline o runtime do Vue em vez de externalizá-lo, certifique-se de que as flags usuais de recurso de compilação do Vue estejam configuradas para esse bundler.
- Trate essas integrações como experimentais e valide-as contra sua própria aplicação antes de lançar.
