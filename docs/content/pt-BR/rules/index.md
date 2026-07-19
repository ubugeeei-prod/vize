---
title: Regras
---

<!-- Generated translation; source: rules/index.md -->

# Regras

Diagnósticos do Vize são documentados como regras, não como uma grande matriz única. Cada página de regra mantém o comportamento de detecção de
próximo dos exemplos de Bad & Good, para que a referência possa ser lida como um manual de regras
ESLint.

## Páginas

- [All Patina rules](./all.md): tabela de metadados de uma página para cada implementação da regra Patina,
  incluindo links de código-fonte do GitHub.
- [Vue rules](./vue.md): estrutura de modelo SFC, diretivas Vue, convenções de componentes e
  verificações de correção do Vue em fila indiana.
- [Type and script rules](./type-and-script.md): Diagnósticos com verificador TypeScript e Vapor
  restrições de roteiro.
- [HTML rules](./html.md): verificações de validade HTML e marcação semântica.
- [Accessibility rules](./accessibility.md): ARIA, interação com teclado, gravadores, pontos de referência e
  checagens de mídia acessíveis.
- [SSR rules](./ssr.md): renderização do servidor e riscos de hidratação.
- [Vapor rules](./vapor.md): Restrições de template apenas para vapor.
- [Ecosystem rules](./ecosystem.md): verificações com presets para Nuxt, Vue Router, Pinia, vue-i18n,
  Vue Test Utils e Void Vue.
- [Musea and CSS rules](./musea-and-css.md): Verificações de blocos de arte Musea e diagnósticos de estilo.
- [Cross-file rules](./cross-file.md): diagnósticos de grafos de projeto emitidos por
  `vize lint --cross-file`.

## Presets

`essential` contém regras de correção que quase sempre devem ser ativadas. `happy-path` adiciona
verificações práticas de higiene para o desenvolvimento diário do Vue. `ecosystem` parte do amplo pacote padrão de
e adiciona Vue Router, Vue I18n, Pinia, Vue Test Utils, Nuxt e verificações Void Vue. `nuxt`
inclui expectativas de SSR orientadas para Nuxt e expectativas para Vapor. `opinionated` é o preset
mais amplo embutido.

`incremental` começa vazio. Use-o quando um host quiser optar por regras específicas sem herdar um preset
maior.

## Configuração Consciente de Tipos

Regras que precisam de informação semântica leem o projeto TypeScript através de `tsconfig.json`. Prefiro
colocar nomes de ambientes compartilhados em `compilerOptions.types` ou referências de projetos em vez de manter
uma lista de `globals` separada na configuração do Vize.
