---
title: Configuração
description: Compartilhe as opções do Vize na configuração Vite e no projeto TypeScript existentes.
---

<!-- Reviewed translation; source: guide/configuration.md -->

# Configuração

Mantenha as opções do Vize em `vite.config.ts` e o projeto TypeScript em `tsconfig.json`. Os valores padrão dispensam um arquivo de configuração específico do Vize.

> [!NOTE]
> A descoberta da configuração Vite pela CLI e pelo editor nativos, assim como o init sem arquivo específico, está em preparação para a próxima versão. Até lá, as ferramentas nativas publicadas ainda usam o formato específico existente para opções compartilhadas personalizadas. A [referência](./configuration-reference.md) descreve esse formato.

## Configuração Vite+

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  compiler: { sourceMap: true },
  lint: { vize: { preset: "essential" } },
  fmt: { vize: { printWidth: 100 } },
  typecheck: { strict: true },
});
```

| Local | Função | Comando |
| --- | --- | --- |
| `compiler` | Compilação Vue | `vp dev`, `vp build` |
| `lint.vize` | Regras lint Vue | `vp run lint` |
| `fmt.vize` | Formatação Vue | `vp run fmt:check` |
| `typecheck` | Verificação de tipos Vue | `vp run typecheck` |
| `pack.vize` | Declarações de biblioteca | `vp run pack` |

Use `vp run check` para as tarefas combinadas do Vize. Os comandos integrados `vp check`, `vp lint` e `vp fmt` mantêm o comportamento próprio do Vite+. Scripts existentes podem renomear as tarefas geradas para `vize:<nome>`; veja [nomes e substituições das tarefas](/guide/vite-plus.md#tasks).

## Alterar uma regra

Coloque as regras Vue em `lint.vize.rules` e as regras do Oxlint em `lint.rules`. Veja os parâmetros em [opções de regras](/rules/options.md) e os exemplos completos no [catálogo](../rules/all.md).

```ts
export default defineConfig({
  lint: {
    vize: { rules: { "vue/no-v-html": "error" } },
    rules: { "no-debugger": "error" },
  },
});
```

<span id="lint-rule-options"></span>

### Opções das regras lint

As [opções das regras](/rules/options.md) mostram os objetos tipados e os exemplos incorretos/corrigidos (em inglês). Campos desconhecidos são rejeitados.

## Escolher os recursos

Defina `compiler`, `typecheck`, `lint.vize` ou `fmt.vize` como `false` para desativar esse recurso. O Vize formata Vue e o Oxfmt os outros arquivos. Veja a [divisão de responsabilidades e conflitos](/guide/vite-plus.md#lint-and-formatter-ownership).

## Vite convencional

Use o plugin para compilar e um objeto `vize` no nível superior para compartilhar opções com a CLI e o editor. A importação do plugin também adiciona os tipos da configuração Vite.

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
  vize: {
    linter: { preset: "essential" },
    formatter: { printWidth: 100 },
    typeChecker: { strict: true },
  },
});
```

<span id="standalone-cli"></span>

## CLI independente

Instale `vize` e execute os comandos na raiz do pacote de destino. A CLI também lê `vite.config.*` e seu projeto TypeScript. As opções Vite+ `compiler`, `typecheck`, `lint.vize` e `fmt.vize` são traduzidas para os comandos nativos. As opções nativas explícitas no objeto `vize` têm prioridade sobre essa tradução.

```bash
vp install -D vize
vp exec vize check
```

A busca para no diretório que contém o `package.json`, `tsconfig.json` ou `jsconfig.json` mais próximo. Em um monorepo, execute os comandos no pacote de destino ou use `vize.entries`, e selecione explicitamente as pastas de trabalho do editor. A descoberta de configurações Vite aninhadas por documento ainda está em desenvolvimento.

Com uma configuração Vite, `build`, `lint`, `fmt` e `check` sem argumento de entrada usam o `root` Vite selecionado. Um `root` relativo parte do diretório da configuração. Os caminhos `typeChecker` do Vite e os `basePath` dos escopos partem dessa raiz; configurações específicas mantêm sua base no diretório da configuração. Arquivos, globs e `--tsconfig` passados explicitamente à CLI continuam relativos ao diretório de execução.

As exclusões globais compartilhadas retiram arquivos da busca da CLI e do lint do editor. Um arquivo excluído aberto no editor ainda recebe diagnósticos de sintaxe, tipos e navegação. A ordem e o significado dos padrões, incluindo a negação `!`, são preservados.

## Configuração específica opcional

Um `vize.config.*` existente continua aceito e tem prioridade sobre a configuração Vite no mesmo diretório. A opção CLI `--config` seleciona um arquivo explicitamente. Opções diretas do plugin e recursos explicitamente definidos no editor prevalecem sobre as opções compartilhadas; `config: false` desativa o carregamento automático pelo plugin.

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

A [referência compartilhada](./configuration-reference.md) descreve a descoberta, os formatos específicos TypeScript/JSON/PKL, as entradas por escopo, a resolução dos tipos Vue e as opções LSP/Musea. Veja a [referência do compilador](./compiler-configuration-reference.md) para opções e sintaxes, e os [recursos experimentais](/guide/experimentals.md) para opções que precisam ser ativadas.

