---
title: Configuração
description: Compartilhe as opções Vue na configuração Vite e no projeto TypeScript existentes.
---

<!-- Reviewed translation; source: guide/configuration.md -->

# Configuração

Mantenha as opções das ferramentas Vue em `vue:` no `vite.config.*`, e as opções do projeto TypeScript em `tsconfig.json`. A CLI, a integração Vite e a extensão do editor leem esses arquivos. Não é necessário um arquivo de configuração específico do Vize.

## Preservar a configuração Vite+

Use o auxiliar do Vize com a configuração existente:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

const vpConfig = {
  server: { port: 3000 },
};

const vueConfig = {
  compiler: { sourceMap: true },
  linter: { preset: "essential" },
  formatter: { printWidth: 100 },
};

export default defineConfig(vpConfig).withVue(vueConfig);
```

`withVue` preserva as opções, os plugins e os hooks existentes do Vite+. Objetos de configuração, funções, promises e funções assíncronas são aceitos. Uma seção inline `vue: { ... }` é equivalente. Veja [configuração Vue compartilhada](/guide/vue-configuration.md) (em inglês) para caminhos, escopos e composição.

## TypeScript e verificação dos templates Vue

Preserve o projeto TypeScript existente, incluindo `extends` e `references` padrão. Coloque as opções dos templates Vue em `vueCompilerOptions`:

```json
{
  "compilerOptions": {
    "strict": true,
    "noEmit": true,
    "jsx": "preserve"
  },
  "vueCompilerOptions": {
    "strictTemplates": true
  },
  "include": ["src/**/*"]
}
```

Projetos que usam `vue-tsc` podem manter sua estrutura TypeScript. A configuração de rigor do TypeScript continua em `compilerOptions`; as opções específicas do Vue pertencem a `vueCompilerOptions`. A [referência](./configuration-reference.md) lista as opções Vue aceitas e as substituições explícitas do Vize.

<span id="lint-rule-options"></span>

## Alterar uma regra

Coloque as regras Vue em `vue.linter.rules` e mantenha as regras do Oxlint em `lint.rules`:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: { rules: { "no-debugger": "error" } },
}).withVue({
  linter: { rules: { "vue/no-v-html": "error" } },
});
```

Veja os parâmetros em [opções de regras](/rules/options.md) e exemplos incorretos e corrigidos no [catálogo](../rules/all.md). Campos de opções desconhecidos são rejeitados.

## Vite convencional

Mantenha a configuração Vite existente e substitua o plugin de compilação Vue:

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
  vue: {
    linter: { preset: "essential" },
    formatter: { printWidth: 100 },
  },
});
```

<span id="standalone-cli"></span>

## CLI e editor

Execute os comandos na raiz do pacote de destino. O editor resolve o projeto de cada documento dentro do espaço de trabalho selecionado e lê suas opções Vite e TypeScript. Cada pacote de um monorepo mantém seu próprio `tsconfig.json`, referências e configuração Vite. Use `--tsconfig` para selecionar outro projeto TypeScript explicitamente.

```bash
vp exec vize check
vp exec vize lint src
vp exec vize fmt --check src
```

Caminhos nativos relativos na configuração Vite usam o `root` Vite selecionado. Um `root` relativo parte do diretório da configuração. Argumentos de arquivo explícitos e `--tsconfig` continuam relativos ao diretório de execução. Flags da CLI e recursos explicitamente definidos no editor prevalecem sobre as opções compartilhadas.

As exclusões globais compartilhadas retiram arquivos da busca da CLI e do lint do editor. Um arquivo excluído aberto no editor ainda recebe diagnósticos de sintaxe, tipos e navegação. A ordem e o significado dos padrões, incluindo a negação `!`, são preservados.

## Migrar configurações removidas

`vize.config.*` e o objeto `vize:` no nível superior não são mais aceitos. Mova as opções nativas para `vue:` em `vite.config.*` e as opções do projeto TypeScript para `tsconfig.json`. Remova os arquivos específicos depois de migrar e verificar o resultado. Um arquivo específico antigo ao lado da configuração Vite é um erro, o mesmo erro de migração de um projeto que tenha apenas a configuração específica. `vize init` e `vize setup` preservam os arquivos originais e explicam como migrar.

Não existe mais um `extends` específico do Vize. Importe objetos JavaScript comuns e passe-os para `defineConfig(vpConfig).withVue(vueConfig)` para reutilizar opções. O `extends` padrão do TypeScript e as referências de projeto continuam funcionando.

<span id="arquivos-de-configuração"></span>
<span id="configuração-do-typescript"></span>
<span id="resolução-do-tipo-de-vue"></span>
<span id="entradas-experimentais-em-flat"></span>
<span id="configuração-pkl"></span>
<span id="configuração-json"></span>
<span id="opções-do-compilador"></span>
<span id="sintaxe-do-template"></span>
<span id="modo-de-saída-jsx-tsx"></span>
<span id="diretivas-por-componente"></span>
<span id="precedência"></span>
<span id="diagnósticos"></span>
<span id="dialeto-vue"></span>
<span id="opções-de-análise-estática"></span>
<span id="opções-de-musea"></span>

## Referência detalhada

A [referência compartilhada](./configuration-reference.md) descreve a descoberta, os escopos, a resolução dos tipos Vue e as opções LSP/Musea. Veja a [referência do compilador](./compiler-configuration-reference.md) para opções e sintaxes, e os [recursos experimentais](/guide/experimentals.md) para opções que precisam ser ativadas.

> [!NOTE]
> A CLI e o editor usam os projetos Vite e TypeScript existentes. Migre as configurações dedicadas removidas antes de excluir seus arquivos.
