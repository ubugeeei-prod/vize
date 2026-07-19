---
title: VS Code
---

<!-- Generated translation; source: integrations/vscode.md -->

# Integração com VS Code

> **⚠️ Trabalho em andamento:** O suporte ao editor da Vize ainda é experimental.

> **Importante:** Para suporte diário ao editor do Vue, continue usando as ferramentas oficiais do idioma do Vue
> (`vuejs/language-tools`) por enquanto. Vize foi projetado para avaliação incremental opt-in.

O repositório contém duas extensões experimentais do VS Code:

- **Vize** — suporte à linguagem Vue com suporte de `vize lsp`
- **Vize Art** — destaque de sintaxe para arquivos Musea `*.art.vue`

Instale-os pelo VS Code Marketplace:

```bash
code --install-extension ubugeeei.vize
code --install-extension vize.vize-art
```

Instale ambos se quiser `*.art.vue` receba suporte para passar o curso, completar, go-to-definition e
referência do Vize, além de realçamento de sintaxe.

## Extensão Vize

A extensão do Vize começa `vize lsp` e pode optar por pacotes de capacidades específicos.
Quando você abre um arquivo do Vue com a extensão ainda desativada, ou sem capacidades ativadas, a extensão agora oferece uma configuração de workspace recomendada com um clique, então o passo, o salto e o diagnóstico não fiquem desativados silenciosamente.
Essa configuração grava `vize.enable`, `vize.lint.enable`, `vize.typecheck.enable`e `vize.editor.enable` para o workspace atual.
Se você configurar manualmente apenas `vize.enable: true`, o Vize também usa os diagnósticos recomendados e o perfil
editor em vez de iniciar um servidor de idiomas vazio.
O item da barra de status do Vize se abre `Vize: Show Status`, que te dá o seletor de perfil, o seletor de
binário, a ação de reiniciar, as configurações e os logs de um só lugar.

### Ponto de Partida Recomendado

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": false,
  "vize.editor.enable": false,
  "vize.formatting.enable": false
}
```

Isso permite diagnósticos de fiapos primeiro, deixando navegação, conclusão e formatação para suas ferramentas
existentes do Vue.

### Configurações Comuns

| Ambientação                  | Propósito                                                         |
| ---------------------------- | ----------------------------------------------------------------- |
| `vize.enable`                | Ativar a extensão e o servidor de idiomas                         |
| `vize.serverPath`            | Sobrescreva o caminho executável `vize`                           |
| `vize.lint.enable`           | Habilitar diagnósticos de lint                                    |
| `vize.typecheck.enable`      | Habilitar diagnósticos conscientes de tipos e recursos de backend |
| `vize.editor.enable`         | Ative o pacote de assistência para editores                       |
| `vize.completion.enable`     | Permitir a conclusão                                              |
| `vize.formatting.enable`     | Ativar a formatação de documentos                                 |
| `vize.definition.enable`     | Ativar a definição de acesso                                      |
| `vize.references.enable`     | Habilitar referências                                             |
| `vize.hover.enable`          | Ativar o hover                                                    |
| `vize.codeActions.enable`    | Ativar correções rápidas para fiapos                              |
| `vize.semanticTokens.enable` | Habilitar tokens semânticos                                       |
| `vize.trace.server`          | Comunicação por traço LSP                                         |

### Comandos Úteis

| Comando                                   | Propósito                                                          |
| ----------------------------------------- | ------------------------------------------------------------------ |
| `Vize: Show Status`                       | Abra o hub de status e de ação de configuração                     |
| `Vize: Enable Recommended Profile`        | Ative fiapos, verificação de digitação e assistência para editores |
| `Vize: Enable Lint-Only Profile`          | Ative diagnósticos enquanto mantém outras ferramentas em uso       |
| `Vize: Select Language Server Executable` | Definir `vize.serverPath` a partir de um seletor de arquivos       |
| `Vize: Disable Language Server`           | Pare o Vize para o alvo de configuração atual                      |
| `Vize: Restart Language Server`           | Reiniciar o servidor de idiomas                                    |
| `Vize: Show Output Channel`               | Extensão de mostrar e logs LSP                                     |

### O que a extensão usa

```text
VS Code
  ↕ Language Server Protocol
vize lsp (vize_maestro)
  → vize_armature
  → vize_croquis
  → vize_patina
  → vize_canon
  → vize_glyph
```

### Instalação a partir do Source ou VSIX

Instale `vp` uma vez a partir do [Vite+ install guide](https://viteplus.dev/guide/install), depois:

```bash
git clone https://github.com/ubugeeei-prod/vize.git
cd vize
cd editors/vscode
vp install -- --ignore-workspace
vp pack
vp exec vsce package --no-dependencies --out dist/vize.vsix
code --install-extension dist/vize.vsix
```

## Vize Art Extension

`Vize Art` fornece destaque de sintaxe para arquivos de `*.art.vue` Musea.
O ID da extensão do Marketplace é `vize.vize-art`.

Reconhece:

- `<art>` blocos de metadados
- `<variant>` blocos
- Seções padrão de Vue `<template>`, `<script>`e `<style>`

## Outros Editores

`vize lsp` segue o Protocolo de Servidor de Linguagem e pode ser usado por editores como Neovim, Helix,
Zed e Emacs.

Exemplo de configuração do Neovim:

```lua
require("lspconfig").vize.setup({
  cmd = { "vize", "lsp" },
  filetypes = { "vue" },
  init_options = {
    lint = true,
    typecheck = true,
    editor = true,
  },
})
```

`editor = true` é a maneira mais fácil de testar o hover, a conclusão, o salto, as referências e os símbolos
juntos. Quando outro servidor TypeScript, como o tsgo, possuir diagnósticos de projeto, mantenha-
`typecheck = false` e ative apenas as capacidades específicas do Vue que você deseja avaliar.
