---
title: Fluxos de Trabalho do Usuário
---

<!-- Generated translation; source: guide/workflows.md -->

# Fluxos de Trabalho do Usuário

Este guia apresenta um caminho compacto pelos fluxos de trabalho comuns do Vize: instale-o, conecte a configuração,
formate, lint, verificação de tipo, compilar e executar as mesmas portas no CI.

## Instalação

Instale o pacote npm no projeto que possui suas dependências do Vue:

```bash
vp install -D vize
```

Para monorepos, instale-o na raiz do workspace quando os pacotes compartilham um lockfile. Instale em um pacote
somente quando esse pacote tiver seu próprio arquivo de bloqueio e grafo de dependências.

## Adicionar Scripts de Pacote

Prefira scripts nomeados a comandos únicos para que execuções locais e de CI compartilhem os mesmos pontos de entrada:

```json
{
  "scripts": {
    "vize:fmt": "vize fmt --check src",
    "vize:fmt:fix": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path --max-warnings 0 src",
    "vize:check": "vize check src",
    "vize:build": "vize build src",
    "vize:ready": "vize ready src"
  }
}
```

`vize ready` é o portão local amplo. Em repositórios maiores, mantenha também os comandos individuais para que
desenvolvedores possam isolar falhas de formatação, lint, verificação de tipos e compiladores.

## Configurar uma vez

Crie `vize.config.ts` na raiz do projeto quando os padrões não forem suficientes:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  formatter: {
    printWidth: 100,
  },
  linter: {
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    tsconfig: "tsconfig.json",
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

Veja [Configuration](./configuration.md) para entradas monorepo planas, PKL, JSON, opções de compilador e detalhes
resolução de tipos Vue.

## Formato

Use o modo de verificação no CI e o modo de escrita localmente:

```bash
vp run vize:fmt
vp run vize:fmt:fix
```

Para trabalhos de migração pontuais, `vize fmt --write` pode direcionar um arquivo, diretório ou globo.

## Fiapos

Comece com `happy-path` para diagnósticos de correção e baixo ruído do Vue:

```bash
vize lint --preset happy-path --max-warnings 0 src
```

Use `--help-level short` quando a saída de CI deve permanecer compacta e `--format json` quando outra ferramenta
consumir o diagnóstico. Veja [CLI](./cli.md) e [Rules](../rules/index.md) para a regra completa
superfície.

## Verificação de Tipo

Execute `vize check` a partir da raiz do projeto para que os `tsconfig`ativos, versão do Vue, pacotes de framework,
e tipos ambient venham do mesmo grafo de dependência:

```bash
vize check src
```

Para verificações monorepo específicas de pacotes, execute a partir do diretório do pacote ou defina `typeChecker.tsconfig`
em uma entrada de configuração com escopo.

## Compilar

Use `vize build` quando precisar de saída do compilador fora do caminho do plugin Vite:

```bash
vize build src --output dist/vize
```

Para aplicações Vite, prefira `@vizejs/vite-plugin` e deixe o Vite cuidar da orquestração da build. Veja
[Vite Plugin](./vite-plugin.md).

## CI

Use os mesmos scripts de pacote em CI:

```yaml
- run: vp install --frozen-lockfile
- run: vp run vize:fmt
- run: vp run vize:lint
- run: vp run vize:check
```

Mantenha `vize:build` no gate apenas quando o projeto consome diretamente a saída do compilador do Vize. Para
aplicações Vite, a build normal do app exercita o plugin.

## Falhas de Depuração

Quando uma falha não está clara:

- reexecuta com `--format json` para inspecionar campos diagnósticos estáveis;
- use `--profile` em `check`, `lint`ou `build` para encontrar fases lentas;
- criar uma carga útil inspetora com `vize inspector` para desajustes do compilador;
- Inclua o menor arquivo `.vue` ou fatia do projeto ao solicitar uma correção.

As páginas [Testing & Feedback](./testing.md) e [Troubleshooting](./troubleshooting.md) cobrem
reportagens, eventos do mundo real e problemas comuns do ambiente.
