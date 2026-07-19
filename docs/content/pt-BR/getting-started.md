---
title: Começando
---

<!-- Generated translation; source: getting-started.md -->

# Começando

> **⚠️ Trabalho em andamento:** O Vize está em desenvolvimento ativo e ainda não está pronto para uso em produção. APIs e limites de pacotes podem mudar sem aviso prévio.

## O que é Vize?

Vize (_/viːz/_) é uma Vue.js cadeia de ferramentas escrita em Rust. O espaço de trabalho contém blocos de construção
compartilhados para:

| Área                      | Caixa(s) principal(es) de ferrugem                                                                                                                                                                                                                                                                                                                                                                                                                                                              | Ponto de entrada voltado para o usuário         |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| Compilação                | [`vize_atelier_core`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_core), [`vize_atelier_dom`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_dom), [`vize_atelier_vapor`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_vapor), [`vize_atelier_ssr`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_ssr), [`vize_atelier_sfc`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_sfc) | `@vizejs/vite-plugin`, npm `vize:build` script  |
| Fiapos                    | [`vize_patina`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_patina)                                                                                                                                                                                                                                                                                                                                                                                                             | O NPM `vize:lint` roteiro, `oxlint-plugin-vize` |
| Formato                   | [`vize_glyph`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_glyph)                                                                                                                                                                                                                                                                                                                                                                                                               | Roteiro `vize:fmt` NPM                          |
| Verificação de tipo       | [`vize_canon`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_canon)                                                                                                                                                                                                                                                                                                                                                                                                               | NPM `vize:check` roteiro                        |
| Suporte ao editor         | [`vize_maestro`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_maestro)                                                                                                                                                                                                                                                                                                                                                                                                           | VS Code, Zed, Rust `vize lsp`                   |
| Ferramentas de arte musea | [`vize_musea`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_musea)                                                                                                                                                                                                                                                                                                                                                                                                               | `@vizejs/vite-plugin-musea`                     |
| Encadernações             | [`vize_vitrine`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_vitrine)                                                                                                                                                                                                                                                                                                                                                                                                           | `@vizejs/native`, `@vizejs/wasm`                |

Este guia recomenda [Vite+](https://viteplus.dev/) (`vp`) para gerenciamento de pacotes em JavaScript e comandos de projeto. Ela mantém o fluxo de instalação e exec consistente entre gerenciadores de pacotes, enquanto ainda usa a ferramenta subjacente do workspace.

Se você ainda não tem `vp` , instale uma vez e abra uma nova carcaça:

```bash
curl -fsSL https://vite.plus | bash
```

Veja o [Vite+ docs](https://viteplus.dev/) e o [Installing Dependencies guide](https://viteplus.dev/guide/install) para mais informações.

## O que o Vize faz

Em um nível geral, o Vize é dividido em algumas rotas reutilizáveis:

| Lane                   | Pacote ou script                         | O que você ganha                                                                                          |
| ---------------------- | ---------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| Compilar               | `@vizejs/vite-plugin`, `vize:build`      | Compilação Vue SFC nativa de ferrugem, saída SSR, modo Vapor, manuseio de CSS com escopo                  |
| Análise estática       | `vize:lint`, `oxlint-plugin-vize`        | Template Vue, script, CSS, a11y, SSR, Vapor, Musea, cross-file e diagnósticos conscientes de tipos        |
| Verificação de tipo    | `vize:check`                             | Geração Virtual TypeScript, diagnóstico de projetos, mapeamento de diagnóstico Vue-to-source              |
| Formato                | `vize:fmt`                               | Formatação SFC do Vue com opções de projeto e CLI                                                         |
| Galeria de componentes | `@vizejs/vite-plugin-musea`, `musea-vrt` | arquivos de arte, variantes de componentes, configuração de pré-visualização, tokens de design, a11y, VRT |
| Suporte ao editor      | VS Code, Zed, Rust `vize lsp`            | Diagnósticos e recursos do editor com opção de participação                                               |

Veja [Static Analysis](./guide/static-analysis.md) para o modelo de verificação de lint e tipo,
[Rules](./rules/index.md) para saída concreta de regras e
[Configuration](./guide/configuration.md) para opções compartilhadas de configuração e compilador.

Autoria de componentes em JSX/TSX em vez de `.vue` SFCs? Veja o guia [JSX & TSX](./guide/jsx.md) —
`.jsx`/`.tsx` componentes do Vue se compilam pela mesma faixa Rust.

## Escolha seu ponto de entrada

### 1. Projetos Vite

Use o plugin Vite se quiser compilação nativa do Vue em um projeto Vite existente.

```bash
vp install -D @vizejs/vite-plugin
```

Instale `vize` como uma dependência direta apenas quando quiser importar ajudantes de configuração compartilhados da
`"vize"` ou adicionar scripts de pacote Vize como `vize:lint` e `vize:check`.

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

Adicionar opções de compilador em `vize.config.ts` quando quiser as mesmas configurações disponíveis para empacotar scripts
e o plugin:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

### 2. Projetos Nuxt

Use o módulo Nuxt quando quiser que o Vize rode dentro do próprio pipeline Vite da Nuxt.

```bash
vp install @vizejs/nuxt
```

Adicione o módulo ao `nuxt.config.ts`:

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

Gerencie seu servidor de desenvolvimento Nuxt normalmente. Os registradores do módulo `@vizejs/vite-plugin` para compilação de
SFC do Vue, preservando as autoimportações Nuxt, componentes, middleware e transformações SSR.

Veja o guia [Nuxt Integration](./integrations/nuxt.md) para a configuração do Musea e notas específicas do Nuxt.

### 3. Scripts de Pacote npm + Configuração Compartilhada

Use o pacote `vize` npm quando quiser utilidades de configuração compartilhadas e comandos nativos disponíveis
scripts de projeto.

```bash
vp install -D vize
```

Scripts recomendados para pacotes:

```json
{
  "scripts": {
    "vize:build": "vize build src",
    "vize:fmt": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path src",
    "vize:check": "vize check src",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:fmt
vp run vize:lint
vp run vize:check
vp run vize:build
vp run vize:ready
```

O comando `vize check` do pacote npm usa o verificador NAPI empacotado e pode emitir declarações de
componentes Vue com `--declaration --declaration-dir dist/types`. Use a CLI do Rust quando precisar de
`check-server`, LSP, gerenciamento de IDE ou diagnósticos de projetos em entradas do Vue, TS, TSX e `.d.ts`.

### 4. CLI Full Rust

A maioria dos fluxos de trabalho de aplicação deve usar os scripts de pacote npm acima. Use o binário Rust quando
precisar da CLI nativa completa hoje: LSP, gerenciamento de IDE, perfilamento ou `check-server`. Para a versão alfa da v1, os canais públicos
suportados são os binários de lançamento do GitHub e o ponto de entrada do Nix; a CLI Rust ainda não
foi publicada pela crates.io.

```bash
nix run github:ubugeeei-prod/vize#vize -- --help
```

```bash
vize build src/**/*.vue
vize fmt --check src
vize lint --profile src
vize check --profile src
vize ready src
vize lsp
```

## Verificação de Tipos Nativa

`vize check` é alimentado pelo `vize_canon`, que agora se apoia em sessões de projeto [`corsa-bind`](https://github.com/ubugeeei/corsa-bind) para diagnósticos nativos de TypeScript. O Vize gera TypeScript virtual para SFCs do Vue, pede ao Corsa diagnósticos conscientes do projeto e então mapeia os resultados de volta para os arquivos originais `.vue`, `.ts`, `.tsx`e `.d.ts`.

Esse caminho ainda está amadurecendo, então a verificação de tipos de editor continua sendo uma opção opcional por enquanto. A
pilha de runtime é o pacote `@typescript/native-preview`, Corsa/corsa-bind é a camada API com a qual o Vize
se comunica, e o executável instalado pela prévia nativa do TypeScript ainda é comumente chamado
`tsgo`. Use `typeChecker.corsaPath`, ou um script de pacote que rode
`vize check --corsa-path /path/to/tsgo`, quando quiser fixar esse tempo de execução.
`typeChecker.tsgoPath` permanece um apelido de compatibilidade obsoleto.

Alvos úteis para scripts de pacotes:

```json
{
  "scripts": {
    "vize:check": "vize check",
    "vize:check:app": "vize check --tsconfig tsconfig.app.json",
    "vize:check:virtual-ts": "vize check --show-virtual-ts src/components/App.vue",
    "vize:check:declarations": "vize check --declaration --declaration-dir dist/types"
  }
}
```

```bash
vp run vize:check
vp run vize:check:app
vp run vize:check:virtual-ts
vp run vize:check:declarations
```

## Compartilhado `vize.config.*`

Os comandos de pacote npm e `@vizejs/vite-plugin` compartilham descoberta de configuração:

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

Configuração do TypeScript:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
  },
  linter: {
    preset: "opinionated",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    corsaPath: "./node_modules/.bin/tsgo",
  },
  formatter: {
    printWidth: 100,
  },
  musea: {
    include: ["src/**/*.art.vue"],
    basePath: "/__musea__",
  },
  lsp: {
    lint: true,
    typecheck: false,
    editor: false,
    formatting: false,
  },
});
```

Configuração PKL:

```pkl
amends "node_modules/vize/pkl/vize.pkl"

linter {
  preset = "opinionated"
}

typeChecker {
  enabled = true
  strict = true
}

lsp {
  lint = true
  typecheck = false
  editor = false
  formatting = false
}
```

Configuração JSON com esquema:

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "linter": {
    "preset": "opinionated"
  }
}
```

## Pacotes

```bash
vp install -D @vizejs/vite-plugin
vp install @vizejs/native
vp install @vizejs/wasm
vp install @vizejs/unplugin
vp install @vizejs/rspack-plugin @rspack/core
vp install @vizejs/nuxt
vp install @vizejs/vite-plugin-musea
vp install @vizejs/musea-mcp-server
vp install -D oxlint oxlint-plugin-vize
```

Notas:

- `@vizejs/vite-plugin` é a integração recomendada para bundlers hoje.
- `@vizejs/unplugin` e `@vizejs/rspack-plugin` ainda são experimentais.
- `@vizejs/native` e `@vizejs/wasm` expõem diretamente as fixações de ferrugem.
- `@vizejs/vite-plugin-musea` fornece a galeria e o fluxo de trabalho do dev-server para o Musea.

## Galeria de Componentes Musea

Use o Musea quando quiser exemplos de componentes nativos do Vue, documentação, tokens, VRT e verificações a11y:

```bash
vp install -D @vizejs/vite-plugin @vizejs/vite-plugin-musea vize
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [
    vize(),
    musea({
      include: ["src/**/*.art.vue"],
      basePath: "/__musea__",
      previewCss: ["src/styles/main.css"],
    }),
  ],
});
```

Execute seu servidor de desenvolvimento Vite e abra `/__musea__`. Veja [Musea](./guide/musea.md) para arquivos de arte, configuração de pré-visualização
, tokens de design, VRT e variantes geradas.

## Integração Oxlint

Execute o diagnóstico do Vue da Vize dentro do Oxlint:

```bash
vp install -D oxlint oxlint-plugin-vize
```

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn"
  },
  "settings": {
    "vize": {
      "preset": "general-recommended",
      "helpLevel": "short"
    }
  }
}
```

Para uso terminal primeiro, prefira:

```bash
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

## Suporte ao Editor

Para a edição diária do Vue, continue usando `vuejs/language-tools` por enquanto.
recursos do editor Vize são projetados para opt-in incremental.

Ponto de partida do VS Code:

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": false,
  "vize.editor.enable": false,
  "vize.formatting.enable": false
}
```

Ponto de partida do Zed:

```json
{
  "languages": {
    "Vue": {
      "language_servers": ["vize", "..."]
    }
  },
  "lsp": {
    "vize": {
      "initialization_options": {
        "lint": true
      }
    }
  }
}
```

## Desenvolvimento Local

As tarefas locais permanecem locais; [CI parity](./contributing.md#common-checks) usa `nix develop .#testbox`.

```bash
nix develop
vp install --frozen-lockfile
vp check
vp fmt
vp dev
vp build
```
