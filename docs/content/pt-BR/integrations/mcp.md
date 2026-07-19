---
title: Servidor MCP
---

<!-- Generated translation; source: integrations/mcp.md -->

# Servidor MCP

> **⚠️ Trabalho em andamento:** O Vize está em desenvolvimento ativo e ainda não está pronto para uso em produção. As capacidades do servidor MCP podem mudar sem aviso prévio.

O Vize oferece um servidor [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) para fluxos de trabalho de desenvolvimento alimentados por IA. O servidor MCP faz a ponte entre sua galeria de componentes (Musea) e os assistentes de IA, permitindo que eles entendam, naveguem e trabalhem com seus componentes do Vue.

## Instalação

Instale `vp` uma vez a partir do [Vite+ install guide](https://viteplus.dev/guide/install), depois adicione o servidor ao seu projeto:

```bash
vp install -D @vizejs/musea-mcp-server
```

## O que é MCP?

O Protocolo de Contexto do Modelo é um padrão aberto para conectar assistentes de IA (como Claude, ChatGPT e outros) a ferramentas de desenvolvimento. Em vez de assistentes de IA adivinhando sobre sua base de código, o MCP oferece acesso estruturado a dados reais de componentes — props, eventos, slots, variantes e documentação.

O servidor MCP do Vize expõe informações de componentes da galeria Musea, então seu assistente de IA tem o mesmo entendimento dos seus componentes que um desenvolvedor navegando pela galeria teria.

## Capacidades

O servidor MCP fornece as seguintes ferramentas para assistentes de IA:

### Descoberta de Componentes

- **Liste todos os componentes** — Navegue por todos os componentes registrados com suas categorias, tags e status
- **Componentes de busca** — Encontre componentes por nome, tag ou descrição
- **Obtenha metadados de componentes** — Recupere informações detalhadas sobre um componente específico

### API de componentes

- **Props** — Definições completas de prop com tipos, padrões e status exigido
- **Eventos** — Eventos emitidos com tipos de carga útil
- **Slots** — Slots nomeados com tipos de prop de slot
- **Expose** — Métodos e propriedades expostos publicamente

### Informações da história

- **Listagem de variantes** — Todas as variantes definidas em arquivos de arte
- **Fonte da variante** — Código modelo para cada variante
- **Variante padrão** — Qual variante é mostrada por padrão

### Design Tokens

- **Listagem de tokens** — Todos os tokens de design do arquivo tokens
- **Categorias de tokens** — Cores, tipografia, espaçamento, pontos de quebra
- **Resolução de tokens** — Tokens semânticos resolvidos para seus valores primitivos

## Configuração

### Com Claude Code

Adicione o servidor MCP à sua configuração de Código Claude:

```json
// .claude/settings.json
{
  "mcpServers": {
    "vize-musea": {
      "command": "vp",
      "args": ["dlx", "@vizejs/musea-mcp-server"]
    }
  }
}
```

### Com o Claude Desktop

Adicione à sua configuração do Claude Desktop MCP:

```json
{
  "mcpServers": {
    "vize-musea": {
      "command": "vp",
      "args": ["dlx", "@vizejs/musea-mcp-server"]
    }
  }
}
```

### Com outros assistentes de IA

Qualquer assistente de IA compatível com MCP pode usar o servidor. O padrão de configuração é o mesmo — aponte o assistente para `vp dlx @vizejs/musea-mcp-server`.

## Casos de Uso

### Descoberta de Componentes

Peça ao seu assistente de IA para encontrar o componente certo:

> "Quais componentes de botão temos? Me mostre as variantes do VFButton."

A IA pode consultar o servidor MCP para encontrar todos os componentes relacionados aos botões, seus props e as variantes disponíveis — e então sugerir o uso correto.

### Geração de Código

Gerar uso de componentes com props corretos:

> "Crie um formulário com nossos componentes VFInput e VFTextarea, incluindo estados de erro de validação."

A IA conhece os nomes exatos dos props, tipos e variantes disponíveis do servidor MCP, gerando código preciso sem alucinar nomes de prop.

### Referência API

Consulte as APIs dos componentes programaticamente:

> "Quais adereços o VFNameBadgePreview aceita? Quais são os valores válidos para o papel de usuário?"

A IA retorna as definições reais de prop do seu código, não palpites genéricos.

### Assistência em Documentação

> "Escreva documentação para nosso componente SponsorGrid com base em seus adereços e variantes."

A IA pode gerar documentação precisa ao inspecionar os metadados reais dos componentes por meio do MCP.

## Como Funciona

```
AI Assistant
  ↕ MCP Protocol (JSON-RPC over stdio)
@vizejs/musea-mcp-server
  ↕ Reads art files and component sources
Your Project (*.art.vue files + components)
```

O servidor MCP:

1. Descobre todos os arquivos `*.art.vue` do seu projeto
2. Analisa eles usando `vize_musea` para extrair metadados dos componentes
3. Expõe os metadados por meio de ferramentas MCP
4. Responde a consultas de assistentes de IA em tempo real
