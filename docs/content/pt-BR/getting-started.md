---
title: Primeiros passos
---

<!-- Generated translation; source: getting-started.md -->

# Primeiros passos

> **⚠️ Em desenvolvimento:** o Vize está em desenvolvimento ativo e ainda não está pronto para uso
> em produção. As APIs e os limites entre pacotes podem mudar sem aviso prévio.

O Vize (_/viːz/_) é uma cadeia de ferramentas Vue.js nativa em Rust. Ele reúne compilação, lint,
formatação, verificação de tipos, diagnósticos no editor e exploração de componentes em um único
workspace, mantendo cada recurso disponível por meio de pacotes e comandos específicos.

| Necessidade                                                             | Ponto de entrada recomendado |
| ----------------------------------------------------------------------- | ---------------------------- |
| Compilar SFCs Vue no Vite                                               | `@vizejs/vite-plugin`        |
| Compilar SFCs Vue no Nuxt                                               | `@vizejs/nuxt`               |
| Executar lint, formatação e verificação de tipos por scripts do projeto | `vize`                       |
| Combinar os diagnósticos do Vize com o Oxlint                           | `oxlint-plugin-vize`         |
| Explorar e testar componentes                                           | `@vizejs/vite-plugin-musea`  |
| Avaliar recursos de editor                                              | VS Code, Zed ou `vize lsp`   |

## Configurar um projeto existente

Execute o inicializador interativo na raiz do projeto:

```bash
vpx vize init
```

O `vpx` faz parte do [Vite+](https://viteplus.dev/guide/install). Instale o Vite+ primeiro se o
comando não estiver disponível no shell.

Antes de escrever qualquer arquivo, o `vize init` detecta Vite, Vite+ ou Nuxt, o gerenciador de
pacotes, TypeScript, o comando de lint ativo e a configuração existente do Vize. Você escolhe quais
partes serão configuradas:

- o plugin do Vite ou o módulo do Nuxt
- o plugin do Oxlint, no arquivo de configuração realmente lido pelo comando de lint ativo
- scripts de projeto para `vize fmt` e `vize check`
- configurações compartilhadas em `vize.config.*`
- uma recomendação de extensão para o VS Code

Visualize todas as alterações propostas em arquivos e dependências sem gravá-las:

```bash
vpx vize init --dry-run
```

Em CI ou outro ambiente não interativo, selecione os recursos explicitamente:

```bash
vpx vize init --yes --lint --bundler --fmt --typecheck --editor
```

Consulte [Project Setup (em inglês)](../guide/init.md) para conhecer as regras de detecção, todas as
opções, as garantias de idempotência e os casos em que o inicializador se recusa deliberadamente a
editar um arquivo.

## Escolher uma configuração manual

Prefira a configuração manual quando precisar preservar uma configuração existente ou adotar uma
parte do Vize por vez:

- [Plugin do Vite](./guide/vite-plugin.md) — compilação nativa de SFCs Vue no Vite
- [Integração com Nuxt](./integrations/nuxt.md) — caminho compatível pelo pipeline Vite do Nuxt
- [Scripts de pacote e CLI](./guide/cli.md) — `vize build`, `fmt`, `lint`, `check`, `ready` e a CLI
  Rust completa

O Vite é a integração recomendada com bundlers. Os pacotes unplugin e Rspack continuam
experimentais; o escopo atual está em [Outros bundlers](./guide/unplugin.md).

## Continuar pelos guias específicos

Esta página é intencionalmente apenas uma orientação. Para detalhes de configuração e integração,
use os guias específicos como fonte de referência:

- [Configuração](./guide/configuration.md) — `vize.config.*`, opções do compilador, verificação de
  tipos e configurações do Musea
- [Análise estática](./guide/static-analysis.md) — modelo de lint e verificação de tipos
- [Documentação de regras](./rules/index.md) — diagnósticos concretos e exemplos
- [Plugin do Oxlint](./guide/oxlint.md) — predefinições, opções e o arquivo de configuração que cada
  comando realmente lê
- [VS Code e outros editores](./integrations/vscode.md) — perfil opcional do editor e configuração LSP
- [JSX e TSX](./guide/jsx.md) — componentes Vue escritos fora de SFCs `.vue`
- [Musea](./guide/musea.md) — exemplos de componentes, documentação, tokens, a11y e VRT

Enquanto a integração do Vize com editores for experimental, continue usando o
[`vuejs/language-tools`](https://github.com/vuejs/language-tools) oficial no desenvolvimento Vue do
dia a dia.
