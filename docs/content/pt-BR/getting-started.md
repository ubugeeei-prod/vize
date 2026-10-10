---
title: Primeiros passos
description: Adicione o Vize ao Vue com sua configuração Vite e seu projeto TypeScript.
---

<!-- Reviewed translation; source: getting-started.md -->

# Primeiros passos

Adicione o Vize ao aplicativo Vue 3 com [Vite+](https://viteplus.dev/guide/install). Mantenha as opções das ferramentas em `vite.config.ts` e o projeto TypeScript em `tsconfig.json`. O Vize está em desenvolvimento; confira o [nível de suporte](./stability.md) antes de adotá-lo.

## 1. Instalar a integração

Em um projeto Vue existente com Vite+ instalado:

```bash
vp install -D @vizejs/vite-plugin
```

A integração usa a versão Vite+ do projeto (0.2.3 ou posterior). Para Vite convencional, veja a [migração do plugin](/guide/migration.md#vite-plugin); para Nuxt, veja a [integração Nuxt](./integrations/nuxt.md).

## 2. Atualizar `vite.config.ts`

A função adiciona o compilador do Vize e as tarefas nativas de verificação. Remova a importação do plugin Vue anterior e `vue()` de `plugins`; mantenha aliases, servidor, testes e outros plugins. O [guia de migração](/guide/migration.md) mostra os exemplos completos antes e depois.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({});
```

## 3. Compilar e verificar o aplicativo

```bash
vp dev
vp build
vp run check
```

`vp run check` combina a verificação de tipos Vue, lint e formatação com Oxlint e Oxfmt. Use tarefas específicas durante o desenvolvimento:

```bash
vp run typecheck
vp run lint
vp run fmt:check
```

Aplique correções com `vp run check -- --fix`. Use `vp run` para as tarefas geradas do Vize: `vp check`, `vp lint` e `vp fmt` executam as ferramentas próprias do Vite+. O Vize preserva scripts existentes e gera `vize:<nome>` em caso de conflito; veja [nomes e substituições](/guide/vite-plus.md#tasks).

## Escolher o próximo passo

- [Configurar uma regra ou opção do compilador](./guide/configuration.md).
- [Migrar ferramentas existentes](/guide/migration.md) com exemplos completos antes e depois.
- [Entender um diagnóstico lint](./rules/all.md) com exemplos Vue incorretos e corrigidos.
- [Explorar os componentes](./guide/ui/index.md), suas receitas e sua API.
- [Visualizar os componentes com Musea](./guide/musea.md).
- [Configurar o editor](/guide/vite-plus-editor.md) com opções nativas compartilhadas.

Os guias que ainda não foram traduzidos abrem em inglês.

> [!NOTE]
> A descoberta da configuração Vite pela CLI e pelo editor nativos, assim como o init sem arquivo específico, está em preparação para a próxima versão. Até lá, as ferramentas nativas publicadas ainda usam o formato específico existente para opções compartilhadas personalizadas. A [referência](./guide/configuration-reference.md) descreve esse formato.

## Usar o Vize sem Vite+

A [CLI independente](./guide/cli.md) oferece `vize lint`, `vize fmt` e `vize check`. Execute na raiz do pacote de destino e mantenha o projeto TypeScript em `tsconfig.json`. Os comandos nativos podem compartilhar o objeto `vize` no nível superior de `vite.config.*`; veja a [configuração da CLI](./guide/configuration.md#standalone-cli) para a busca e seus limites atuais.

Confira o plano interativo com `vpx vize init --dry-run` e execute `vpx vize init` para escolher os recursos. O [guia de configuração inicial](/guide/init.md) explica a detecção e os limites de edição. O init usa as opções do projeto e os valores padrão, sem criar uma configuração específica do Vize.

<span id="configurar-um-projeto-existente"></span>
<span id="escolher-uma-configuração-manual"></span>
<span id="continuar-pelos-guias-específicos"></span>
