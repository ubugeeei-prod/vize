---
layout: entry
title: Vize
description: Cadeia de ferramentas Vue.js de alto desempenho em ferrugem. Compile, lint, formate, verifique os tipos e explore os componentes do Vue.
hero:
  name: Vize
  text: Cadeia de Ferramentas Vue.js de Alto Desempenho na Ferrugem
  tagline: /viːz/ — Uma ferramenta sábia que enxerga através do seu código. Compile, faça fia, formate, cheque de tipos e explore componentes do Vue — tudo alimentado por Rust. ⚠️ Ainda não está pronto para produção.
  image:
    src: logo.svg
    alt: Vize Logo
  actions:
    - theme: brand
      text: Comece
      link: pt-BR/getting-started.md
    - theme: alt
      text: GitHub
      link: https://github.com/ubugeeei-prod/vize
    - theme: alt
      text: Playground
      link: https://vizejs.dev/play
features:
  - title: Vite Plugin
    details: "Comece pela integração recomendada para aplicações Vue: compilação nativa de SFC dentro do Vite com configuração compartilhada do Vize."
    link: pt-BR/guide/vite-plugin.md
  - title: Pipeline de Análise Estática
    details: Parser, análise semântica, regras de lint, TypeScript virtual, verificações entre arquivos e diagnósticos de editores compartilham as mesmas camadas de análise nativa do Rust.
    link: pt-BR/guide/static-analysis.md
  - title: Documentação das regras
    details: Navegue por Vue, HTML, SSR, Vapor, Musea, type-aware e faça diagnósticos cruzados com exemplos bons e ruins.
    link: pt-BR/rules/index.md
  - title: Configuração Compartilhada
    details: Configure opções do compilador, escaneamento Vite, predefinições de lint, verificação de tipos, formatação, recursos LSP e Musea a partir de `vize.config.*`.
    link: pt-BR/guide/configuration.md
  - title: Verificação de Tipos Nativa
    details: |
      `vize:check` scripts de pacote são executados por sessões de projeto `vize_canon` e Corsa apoiadas por `corsa-bind`, mantendo diagnósticos conscientes do Vue em um caminho nativo.
    link: pt-BR/guide/static-analysis.md
  - title: Pacotes de Scripts e Referência de CLI
    details: Use o pacote npm dos scripts de projeto para fluxos de trabalho de aplicativos, com a CLI Rust documentada para LSP, perfil e uso direto binário.
    link: pt-BR/guide/cli.md
  - title: Inspetor de Compiladores
    details: Inspecionar a saída do Vue, saída do Vize, Virtual TS, VIR e gráficos de arquivos cruzados, depois compartilhe repros permalinked ou relatórios de agentes.
    link: pt-BR/guide/compiler-inspector.md
  - title: Oxlint Plugin
    details: Execute os diagnósticos do Vue da Vize dentro do Oxlint e combine com as regras JS e TS da OXC em uma única passada.
    link: pt-BR/guide/oxlint.md
  - title: Integrações experimentais de bundlers
    details: Existem rollup, webpack, esbuild e um caminho dedicado para o Rspack, mas o Vite continua sendo a integração recomendada e mais estável.
    link: pt-BR/guide/unplugin.md
  - title: 8,3x mais rápido
    details: Compilação multithread de 15.000 arquivos SFC (36,9 MB) em menos de 500ms. Alocação de arena, paralelismo Rayon, zero GC.
    link: pt-BR/architecture/performance.md
  - title: Galeria de Componentes
    details: Musea — arquivos de arte, documentação, geração de paleta, ferramentas a11y e VRT, com o fluxo de trabalho da galeria fornecido por @vizejs/vite-plugin-musea.
    link: pt-BR/guide/musea.md
  - title: Encadernações WASM
    details: Execute o compilador Vue diretamente no navegador com WebAssembly. Playgrounds de energia, documentos e ferramentas educacionais.
    link: pt-BR/guide/wasm.md
  - title: Integração com IA
    details: Servidor MCP permitindo que assistentes de IA entendam e trabalhem com seus componentes do Vue através do Musea.
    link: pt-BR/integrations/mcp.md
  - title: Modo Vapor
    details: Suporte de primeira classe para o modo Vapor do Vue 3.6 — compilação reativa detalhada sem o DOM virtual.
    link: pt-BR/architecture/overview.md
  - title: Filosofia
    details: Arquitetura inspirada na arte, ecossistema de oxidação (OXC, oxlint, corsa-bind) e uma visão unificada de cadeias de ferramentas.
    link: pt-BR/philosophy.md
  - title: Blog
    details: Notas de lançamento para mudanças enviadas, além de notas irregulares para atualizações de design, devlogs e pensamento de projetos.
    link: pt-BR/blog/index.md
---

<!-- Generated translation; source: index.md -->

## Direção Atual

Uma das maiores mudanças recentes no Vize é a verificação de tipos nativos. O comando `vize check` usado pelos scripts de
npm e pelo pipeline de verificação de tipos voltado para o editor estão migrando para `vize_canon` plus
[`corsa-bind`](https://github.com/ubugeeei/corsa-bind), o que permite que o Vize mantenha arquivos virtuais do Vue e
diagnósticos do projeto TypeScript em um caminho nativo por mais tempo.

Isso importa mais do que a velocidade bruta. Isso oferece ao Vize um ciclo mais fechado entre análise de templates, diagnósticos, navegação e recursos futuros do editor, ao mesmo tempo em que reduz a quantidade de trabalho que precisa ser retornada por um processo de compilador hospedado em JavaScript. A história da fidelidade ainda está alcançando, mas essa é claramente a direção que a cadeia de ferramentas está tomando.

A mesma direção se aplica ao linting e à musea. A análise estática começa com o parser e o Croquis
modelo semântico, depois alimenta as regras de lint do Patina, o TypeScript virtual da Canon, decisões do compilador, diagnósticos de
do editor e metadados da galeria de componentes. O fluxo prático está documentado em
[Static Analysis](./guide/static-analysis.md), com detalhes de configuração em
[Configuration](./guide/configuration.md). A regra concreta e o catálogo diagnóstico estão em
[Rules](./rules/index.md).

## Autor

![ubugeeei](https://github.com/ubugeeei.png)

- \*[ubugeeei](https://github.com/ubugeeei)\*\* é engenheira de software baseada em Tóquio, atuando em Vue, Rust, design e ferramentas de linguagem.

Ele faz parte da [Vue.js Core Team](https://vuejs.org/about/team.html) [Vue.js Japan User Group](https://github.com/vuejs-jp) equipe central, contribuidor [Vite+](https://github.com/voidzero-dev/vite-plus) núcleo e engenheiro-chefe da [mates-dev](https://github.com/mates-dev).

Ele também é o criador de [chibivue](https://github.com/chibivue-land/chibivue), [Vize](https://github.com/ubugeeei-prod/vize)e [Ox Content](https://github.com/ubugeeei/ox-content).

- GitHub: [github.com/ubugeeei](https://github.com/ubugeeei)
- X (Twitter): [@ubugeeei](https://x.com/ubugeeei)
- Blog: [wtrclred.io](https://wtrclred.io)
- chibivue.land: [chibivue.land](https://chibivue.land)

## Patrocinador

Vize é um projeto gratuito e de código aberto licenciado pelo MIT. Desenvolver e manter uma cadeia de ferramentas completa — compilador, linter, formatador, verificador de tipos, LSP, galeria de componentes e bindings WASM — é um esforço significativo que exige foco e dedicação sustentados.

Se o Vize economiza seu tempo, melhora sua experiência de desenvolvimento ou você acredita na visão de uma cadeia de ferramentas Vue.js de alto desempenho, por favor, considere patrocinar o projeto:

- A infraestrutura do CI/CD runner é patrocinada pela [Blacksmith](https://www.blacksmith.sh/).
- [GitHub Sponsors](https://github.com/sponsors/ubugeeei)

Seu apoio ajuda a financiar o desenvolvimento contínuo, custos de infraestrutura e garante que a Vize continue sendo gratuita para todos. Cada contribuição — independentemente do tamanho — faz uma diferença real.
