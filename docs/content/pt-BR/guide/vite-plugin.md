---
title: Vite Plugin
---

<!-- Generated translation; source: guide/vite-plugin.md -->

# Vite Plugin

> **⚠️ Trabalho em andamento:** O Vize está em desenvolvimento ativo e ainda não está pronto para uso em produção. Teste cuidadosamente antes de adotar em projetos não triviais.

> **Status do bundler:** `@vizejs/vite-plugin` atualmente é a integração de bundler mais estável.
> Para rollup, webpack e esbuild, use `@vizejs/unplugin`, e para rspack use `@vizejs/rspack-plugin`.
> Esses caminhos não-Vite ainda são instáveis e devem ser tratados como experimentais.

`@vizejs/vite-plugin` fornece compilação nativa de Vue SFC para projetos Vite. Ele foi projetado como um **substituto direto** para `@vitejs/plugin-vue` — seus componentes existentes do Vue funcionam sem modificações.

## Instalação

Instale `vp` uma vez a partir do [Vite+ install guide](https://viteplus.dev/guide/install), depois adicione os pacotes:

```bash
vp install -D @vizejs/vite-plugin
```

Adicione `vize` como dependência direta apenas se seu projeto importar ajudantes de configuração compartilhados de `"vize"`
ou expor scripts de pacote como `vize:lint` e `vize:check`.

## Uso Básico

```javascript
// vite.config.js
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

É isso. Substitua `@vitejs/plugin-vue` por `@vizejs/vite-plugin` e seu projeto será compilado pelo Rust.

## Importações do TypeScript Vue

Adicione o pacote de plugins ao `compilerOptions.types` para tornar as importações `.vue` diretas resolvíveis pelo
TypeScript sem escrever um `env.d.ts` local de shim:

```json
{
  "compilerOptions": {
    "types": ["vite/client", "@vizejs/vite-plugin"]
  }
}
```

Isso não exige adicionar `vize` como uma dependência direta do projeto.

Para projetos Vite Plus, mantenha o tipo cliente Vite Plus e anexe o pacote de plugins:

```json
{
  "compilerOptions": {
    "types": ["vite-plus/client", "@vizejs/vite-plugin"]
  }
}
```

Para a maioria dos projetos, mantenha as opções de plugins diretas pequenas e coloque configurações estáveis do compilador em
`vize.config.ts`.

## Configuração Compartilhada

O ponto de entrada compartilhado recomendado é `vize`. Um único arquivo `vize.config.*` é lido tanto pelos comandos npm
package quanto pelo `@vizejs/vite-plugin`.

```bash
vp install -D vize
```

Arquivos de configuração suportados:

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

Configuração do TypeScript:

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

Configuração PKL:

```pkl
amends "node_modules/vize/pkl/vize.pkl"

compiler {
  sourceMap = true
}

vite {
  scanPatterns = new Listing {
    "src/**/*.vue"
  }
}
```

Configuração JSON com esquema:

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "vite": {
    "scanPatterns": ["src/**/*.vue"]
  }
}
```

Importar `defineConfig` do `@vizejs/vite-plugin` ainda funciona para compatibilidade retroativa, mas `import { defineConfig } from "vize"` é o caminho compartilhado daqui para frente.

Veja [Configuration](./configuration.md) para a configuração compartilhada completa.

Projetos Vite Plus primeiro também podem manter as configurações apenas de startup ativadas em `vite.config.ts`:

```ts
import { defineConfig } from "vite-plus";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    vize({
      config: {
        compiler: {
          sourceMap: true,
          vapor: false,
        },
        vite: {
          scanPatterns: ["src/**/*.vue"],
        },
        musea: {
          include: ["src/**/*.art.vue"],
        },
      },
    }),
  ],
});
```

A configuração inline está disponível para o plugin Vite e para a loja compartilhada de plugins durante a execução do Vite Plus.
Use `vize.config.*` para configurações que também devem ser lidas por comandos CLI e LSP.

## Opções do compilador

Opções diretas passaram para `vize()` sobreposição `vize.config.*`.
A precedência completa são opções de plugin direto, depois `config`em linha, depois `vize.config.*`, e depois
padrão.

```ts
vize({
  vueVersion: 3,
  sourceMap: true,
  ssr: false,
  vapor: false,
  customRenderer: false,
  templateSyntax: "standard",
  scanPatterns: ["src/**/*.vue"],
  ignorePatterns: ["node_modules/**", "dist/**", ".git/**"],
});
```

| Opção                  | Onde configurá-lo                                       | Descrição                                                                                                                                                  |
| ---------------------- | ------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `vueVersion`           | `vize({ vueVersion })`                                  | Defina `0.11`, `1`, `2`ou `"legacy"` para rodar em modo de compatibilidade legado não invasivo com o Vue e deixar a compilação SFC para o compilador host. |
| `sourceMap`            | `compiler.sourceMap` ou `vize({ sourceMap })`           | Gerar mapas de fonte. O padrão é desenvolvimento ligado, produção desligado.                                                                               |
| `ssr`                  | `compiler.ssr` ou `vize({ ssr })`                       | Forçar a compilação do SSR quando a flag de build do SSR do Vite não é suficiente.                                                                         |
| `vapor`                | `compiler.vapor` ou `vize({ vapor })`                   | Compilar templates pelo backend do Vapor.                                                                                                                  |
| `jsxMode`              | `compiler.jsxMode` ou `vize({ jsxMode })`               | Backend de saída padrão (`"vdom"` / `"vapor"`) para componentes `.jsx`/`.tsx` . Diretivas `"use vue:*"` por componente prevalecem sobre isso.              |
| `customRenderer`       | `compiler.customRenderer` ou `vize({ customRenderer })` | Trate tags minúsculas que não sejam HTML como elementos personalizados de renderização. Não corresponde a tags PascalCase como `<TresMesh>`.               |
| `customElements`       | `compiler.customElements` ou `vize({ customElements })` | Padrões de tag compilados como custom elements. Use `["Tres*"]` para tags PascalCase do TresJS.                                                            |
| `templateSyntax`       | `compiler.templateSyntax` ou `vize({ templateSyntax })` | Escolha `"standard"`, `"strict"`ou `"quirks"` tratamento de sintaxe de template.                                                                           |
| `include`              | `vite.include` ou `vize({ include })`                   | Arquivos que o plugin deve compilar.                                                                                                                       |
| `exclude`              | `vite.exclude` ou `vize({ exclude })`                   | Arquivos que o plugin deveria ignorar.                                                                                                                     |
| `scanPatterns`         | `vite.scanPatterns` ou `vize({ scanPatterns })`         | Padrões glob usados para pré-compilação de inicialização.                                                                                                  |
| `ignorePatterns`       | `vite.ignorePatterns` ou `vize({ ignorePatterns })`     | Os padrões glob pulavam durante a pré-compilação de inicialização.                                                                                         |
| `configMode`           | `vize({ configMode })`                                  | Use `"root"`, `"auto"`ou `false` para carregamento de configuração compartilhada.                                                                          |
| `configFile`           | `vize({ configFile })`                                  | Carregue um arquivo de configuração específico.                                                                                                            |
| `config`               | `vize({ config })`                                      | Configuração compartilhada inline para as configurações de runtime do Vite Plus.                                                                           |
| `handleNodeModulesVue` | `vize({ handleNodeModulesVue })`                        | Compilar `.vue` arquivos importados de `node_modules` sob demanda.                                                                                         |
| `debug`                | `vize({ debug })`                                       | Imprimir logs de depuração do plugin.                                                                                                                      |

Receitas comuns:

```ts
// Vapor-oriented build
vize({ vapor: true });

// Tags PascalCase do TresJS
vize({
  customRenderer: true,
  customElements: ["Tres*", "primitive"],
});

// Existing templates that rely on parser edge cases, such as
// v-for alias edge parens or `<div />` as a self-closing leaf
vize({ templateSyntax: "quirks" });

// Monorepo package with explicit scan roots
vize({
  root: import.meta.dirname,
  scanPatterns: ["src/**/*.vue", "examples/**/*.vue"],
});

// Legacy Vue / Nuxt 2 Bridge project with an existing host compiler plugin
vize({ vueVersion: 2 });
```

`vueVersion: 0.11`, `1`, `2`e `"legacy"` são modos de compatibilidade host-compilador. O Vize não compila
arquivos de `.vue` nesses modos, não expõe o shim da API `vite:vue` do Vue 3 e não injet
a flags de funcionalidades do bundler do Vue 3. Mantenha o plugin existente do compilador do Vue, `vue-loader`, ou o compilador
próprio do Nuxt 2 configurados normalmente.

## Como Funciona

O plugin intercepta `.vue` solicitações de arquivo e as compila usando o pipeline Rust-native do Vize por meio de Node.js bindings NAPI:

1. **Pré-compilação** — Às `buildStart`, o plugin descobre todos os arquivos `.vue` e os compila em lote usando `compileBatch`. Isso aciona a compilação paralela baseada em Rayon no lado Rust, processando todos os arquivos em todos os núcleos de CPU simultaneamente.

2. **Compilação sob demanda** — Durante o desenvolvimento, se um arquivo `.vue` for solicitado que não está no cache (por exemplo, importado dinamicamente), ele é compilado em tempo real via `compileFile`.

3. **HMR** — Quando um arquivo `.vue` muda, apenas esse arquivo é recompilado. O plugin detecta se a mudança é apenas de estilo e aplica uma atualização de HMR apenas de estilo sempre que possível, evitando uma re-renderização completa do componente.

4. **Extração CSS** — Em construções de produção, todo o CSS com escopo dos componentes do Vue é extraído e fundido em `assets/vize-components.css`, eliminando o overhead de injeção no estilo por componente.

### Pipeline de Compilação

```
.vue file
  → Armature (Parser)          — Tokenizes and parses the SFC structure
  → Croquis (Semantic Analysis) — Analyzes template expressions and bindings
  → Atelier (Compilation)       — Generates optimized JavaScript output
  → Vitrine (NAPI Binding)      — Delivers the result to Node.js
  → Vite module graph            — Served as a virtual module
```

A mesma camada de análise semântica é reutilizada por linting e verificação de tipos. Veja
[Static Analysis](./static-analysis.md) para o lado diagnóstico do pipeline.

## Comparação

| Característica          | @vitejs/plugin-vue | @vizejs/vite-plugin                    |
| ----------------------- | ------------------ | -------------------------------------- |
| Idioma                  | JavaScript         | Ferrugem (NAPI)                        |
| Compilação SFC          | Sim                | Sim                                    |
| Compilação de Modelos   | Sim                | Sim                                    |
| Configuração do Script  | Sim                | Sim                                    |
| Escopo CSS              | Sim                | Sim                                    |
| Suporte à SSR           | Sim                | Sim                                    |
| HMR                     | Sim                | Sim (otimização apenas de estilo)      |
| Pré-compilação por lote | Não                | Sim (paralelo via Rayon)               |
| Extração CSS            | Por componente     | Fila única fundida                     |
| Modo Vapor              | Experimental       | Primeira classe (`vize_atelier_vapor`) |

## Recursos Avançados

### Pré-compilação por lote

Diferente do `@vitejs/plugin-vue`, que compila cada arquivo `.vue` na primeira solicitação, o Vize pré-compila todos os arquivos de `.vue` descobertos no início da build usando compilação em lote multithread. Isso significa:

- **Inicialização do servidor de desenvolvimento** — Todos os componentes estão prontos antes do primeiro carregamento da página
- **Construções de produção** — Paralelismo máximo desde o início

### Reescrita de Ativos Estáticos

O plugin reescreve automaticamente URLs de ativos estáticos em templates. Por exemplo:

```vue
<template>
  <img src="./logo.png" />
</template>
```

O atributo `src` é elevado a uma declaração de importação, permitindo que o Vite processe o ativo através de seu pipeline de ativos (hash, otimização, etc.).

### Defina Substituição

O Vite normalmente pula `import.meta.*` substituto para módulos virtuais (prefixado com `\0`). O plugin do Vize aplica manualmente os substitutos define para garantir que valores de `import.meta.env.*` funcionem corretamente nos componentes compilados do Vue.

### Isolamento por ambiente

Para compatibilidade com Nuxt, o plugin isola `define` valores por ambiente Vite (cliente vs. servidor/SSR). Isso impede que valores do ambiente do lado do cliente vazem para a saída do SSR.

## Compatibilidade Nuxt

O plugin expõe um shim de compatibilidade para ferramentas que sondam a API do `@vitejs/plugin-vue`(como o Nuxt). Isso significa que o Vize funciona com a integração embutida do Vue da Nuxt sem configuração especial:

```ts
// nuxt.config.ts — using the dedicated Nuxt module
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

Veja [Nuxt Integration](../integrations/nuxt.md) para mais detalhes.

## Notas

- O plugin requer `@vizejs/native` para Node.js bindings NAPI (instalados automaticamente como dependência)
- A compilação do modo vapor está disponível via `vize_atelier_vapor` (Vue 3.6+)
- A compilação VDOM usa `vize_atelier_dom`
- O plugin suporta `virtual:vize-styles` para importar todo o CSS compilado como um módulo
- `.jsx`/`.tsx` Componentes do Vue são compilados automaticamente pelo mesmo plugin — veja o guia [JSX & TSX](./jsx.md)
- Para suporte experimental a rollup / webpack / esbuild / rspack, veja [Experimental Bundler Integrations](./unplugin.md)
