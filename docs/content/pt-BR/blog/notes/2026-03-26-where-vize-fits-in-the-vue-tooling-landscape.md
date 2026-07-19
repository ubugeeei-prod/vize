---
title: Mapa de Ferramentas Vue
description: Um mapa de onde o Vize está localizado na atual paisagem de ferramentas do Vue e como ele difere de projetos adjacentes.
---

<!-- Generated translation; source: blog/notes/2026-03-26-where-vize-fits-in-the-vue-tooling-landscape.md -->

# Mapa de Ferramentas Vue

<div class="blog-post-meta">
<span class="blog-meta-chip">
<span>
<span class="blog-meta-label">Publicado em</span>
<span class="blog-meta-value">26-03-2026</span>
</span>
</span>
<a class="blog-author-card" href="https://github.com/ubugeeei">
<img src="https://github.com/ubugeeei.png" alt="ubugeeei" />
<span class="blog-author-text">
<span class="blog-meta-label">Autor</span>
<span class="blog-meta-value">ubugeeei</span>
</span>
</a>
</div>

Uma razão pela qual o Vize é fácil de entender errado é que ele se sobrepõe a várias ferramentas que as pessoas já conhecem, mas nem sempre na mesma camada.

Alguns desses projetos são oficiais. Alguns são agnósticos em relação ao framework. Alguns são editores em primeiro lugar. Alguns são focados no compilador. Alguns são principalmente sobre verificação de tipos. Alguns estão tentando se tornar uma cadeia de ferramentas completa.

Então, a pergunta mais útil não é "qual deles é melhor?" É: **qual problema cada ferramenta realmente está tentando resolver?**

## A Versão Curta

Aqui está a maneira mais rápida de posicioná-los:

| Projeto                          | Centro principal de gravidade                                                                                                           | O que não é                                                         |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| **Vize**                         | Cadeia de ferramentas independente completa do Vue em Rust                                                                              | Não é a pilha oficial do editor do Vue                              |
| **Ferramentas de Linguagem Vue** | Editor oficial do Vue + ferramentas de verificação de tipos                                                                             | Não é uma cadeia completa de compiladores/linter/formateadores      |
| **Golar**                        | `typescript-go`framework de verificação de tipos em linguagem embutida baseado em linguagem                                             | Não é uma cadeia de ferramentas específica para a Vue               |
| **Verter**                       | Compilador completo alternativo de Vue + LSP + toolchain de compilação                                                                  | Não é a cadeia oficial de ferramentas do Vue                        |
| **Vite+**                        | Ponto de entrada unificado para desenvolvimento web em runtimes, gerenciamento de pacotes, desenvolvimento/construção/verificação/teste | Não é um compilador ou linter específico do Vue                     |
| **Oxlint**                       | Linter JS/TS de alto desempenho                                                                                                         | Não é uma pilha completa de fiapos que conheça o modelo Vue sozinha |

Se você mantiver essa tabela na cabeça, a maior parte da confusão desaparece.

## Vize

Vize é melhor entendido como uma **cadeia de ferramentas independente e completa do Vue em Rust**.

Sua ambição é ampla:

- compilar SFCs do Vue
- Padrões específicos de fiapos da Vue
- formatar arquivos Vue
- type-check de templates Vue e vinculações de script
- alimentar um LSP
- fornecer uma galeria de componentes
- expor ferramentas conscientes do Vue a fluxos de trabalho de IA

Essa amplitude é o que diferencia a Vize da maioria dos projetos nesta comparação. Não é apenas uma integração com editores, nem apenas um verificador de tipos, nem apenas um plugin de bundler. Está tentando ser uma cadeia de ferramentas coerente nativa do Vue, com um centro arquitetônico.

É também por isso que a recente orientação de verificação de tipos é importante. A Vize não está apenas tentando "fazer `vue-tsc` mais rápido." A direção atual é manter a geração de arquivos virtuais conscientes do Vue, o mapeamento de diagnósticos e as informações voltadas para editores dentro de `vize_canon`, com sessões nativas de projeto alimentadas por [`corsa-bind`](https://github.com/ubugeeei/corsa-bind).

## Como a Vize está se aproximando da `tsgo`

Uma nota recente, [`corsa-bind: The Idea of Language Processor Orchestration`](https://wtrclred.io/posts/17), argumenta que a parte interessante não é apenas a execução mais rápida, mas também "mudar a forma do trabalho, não o compilador."

Esse enquadramento é muito próximo de como a Vize está abordando `tsgo`.

Vize não está tentando transformar `tsgo` em toda a história do produto, nem tratando como uma CLI one-shot que é reexecutada para cada longa. A direção está mais próxima de tratar o processamento do TypeScript como um serviço nativo reutilizável dentro de uma cadeia de ferramentas mais ampla do Vue:

- `vize check` materializa um projeto virtual TypeScript compatível com o Vue, abre uma sessão de projeto Corsa e solicita diagnósticos em lote.
- `vize_maestro` pode manter uma ponte Corsa para passar o curso, completar, definir, referenciar e renomear quando a verificação nativa de tipos estiver ativada.
- `vize_patina` usa sessões nativas preguiçosas de Corsa para regras de lint conscientes de tipos, sondando apenas os tipos necessários em vez de reconstruir tudo em uma pilha hospedada em JavaScript.
- `vize_canon` mantém a propriedade da geração de arquivos virtuais específicos do Vue e do mapeamento de código-fonte, enquanto `corsa-bind` e `tsgo` respondem às perguntas do lado do TypeScript.

Então, a `tsgo` história da Vize não é apenas "trocar `vue-tsc` por um binário mais rápido." É mais próximo de construir uma camada de controle nativa do Vue em torno de um processador TypeScript residente, e depois reutilizar essa camada em checagens em lote, recursos do editor e linting consciente de tipos.

## Ferramentas de Linguagem Vize vs Vue

O projeto oficial [Vue Language Tools](https://github.com/vuejs/language-tools) é o editor Vue pronto para produção e a pilha de verificação de tipos. Inclui:

- a extensão **do Vue (Oficial)** VS Code
- `vue-tsc`
- `@vue/language-server`
- `@vue/language-core`

Essa pilha é fundamentalmente sobre **ferramentas de linguagem**: suporte a editores, verificação de tipos, geração virtual de código e integrações que fazem o Vue parecer de primeira classe em IDEs.

Vize se sobrepõe a esse mundo porque também tem um verificador de tipos e um LSP. Mas a Vize está tentando cobrir mais terreno:

- Vize inclui suas próprias ambições de compilador
- Vize inclui ambições de linting e formatação
- Vize inclui superfícies de produto como ferramentas Musea e MCP
- Vize é Rust-first em vez de TypeScript-first

Portanto, a distinção mais simples é:

- **O Vue Language Tools** é o editor oficial e fundação de verificação de tipos para o Vue
- **Vize** é uma tentativa independente de unificar muito mais da cadeia de ferramentas do Vue sob uma única arquitetura Rust

Se sua prioridade é o suporte ao editor pronto para produção hoje, a stack oficial do Vue é a base. Se seu interesse é uma caixa de ferramentas mais ampla, experimental e nativa da Rust, é aí que o Vize começa a fazer sentido.

## Vize vs Golar

[Golar](https://github.com/auvred/golar) não é realmente "outra cadeia de ferramentas do Vue" no mesmo sentido.

Golar se descreve como um framework de linguagem embutida baseado em `typescript-go`. Para o Vue especificamente, ele reutiliza a maquinaria oficial do `@vue/language-core` e foca em fazer linguagens baseadas em extensões como `.vue`, `.astro`e `.svelte` funcionarem com `tsgo`.

Isso significa que o centro de gravidade de Golar é:

- Verificação de tipos de CLI
- Declaração emissora
- `tsgo` integração para linguagens embarcadas
- Infraestrutura de plugins para geração de código virtual

Vize é diferente em dois aspectos importantes:

1. **Escopo**

Golar é principalmente uma história de checagem de tipos e código virtual em torno de `typescript-go`.
Vize está tentando possuir uma fatia muito maior da cadeia de ferramentas do Vue: compilador, linter, formatador, verificador de tipos, LSP, galeria e mais.

2. **Propriedade da camada Vue**

O Golar reutiliza deliberadamente as ferramentas oficiais do Vue para geração de código do Vue.
Vize está tentando construir mais do stack específico da Vue em Rust.

Também começa a aparecer uma diferença prática na camada de execução. O Golar está intimamente associado à integração `typescript-go` para linguagens embarcadas. O caminho nativo atual de verificação de tipos do Vize está sendo moldado em torno de `vize_canon` mais `corsa-bind`, o que torna a questão menos "como reutilizar a pilha oficial com um motor TS mais rápido?" e mais "quanto da cadeia de ferramentas do Vue pode estar dentro de uma arquitetura nativa?"

Assim, o Golar está mais próximo de "fazer `tsgo` funcionarem bem para linguagens embarcadas", enquanto o Vize está mais próximo de "construir uma cadeia nativa de ferramentas do Vue de ponta a ponta."

## Vize vs Verter

[Verter](https://github.com/pikax/verter) provavelmente é o vizinho filosófico mais próximo dessa lista.

Assim como Vize, Verter mira alto. Sua visão pública é um compilador híbrido Rust + TypeScript Vue, LSP, ferramenta de compilação, linter e uma cadeia de ferramentas mais ampla. Isso o coloca na mesma família geral do Vize: ambicioso, full-stack e disposto a repensar a caixa de ferramentas do Vue em vez de corrigir apenas uma camada.

É aí que as diferenças se tornam mais sobre a forma e arquitetura do produto do que sobre categoria:

- **Verter** se apresenta como uma linguagem Vue e uma cadeia de ferramentas de compiladores com uma forte história de VS Code e provedor TS.
- **O Vize** se apresenta como uma cadeia de ferramentas independente de alto desempenho do Vue, com uma interface unificada de CLI, integração com Vite, Musea e uma narrativa mais forte de "um parser / um AST / uma cadeia de ferramentas".

Há também uma diferença de ênfase:

- Verter destaca geração de TSX digitada, backends de Provedores de Tipos como TSGO / tsserver, e um amplo catálogo de regras de lint embutido.
- O Vize destaca uma cadeia de ferramentas unificada nativa de Rust em compilação, lint, formatação, verificação de tipos, ferramentas de editor, galeria de componentes e integração com IA, enquanto se posiciona explicitamente como complementar a ferramentas do ecossistema como Vite+ e Oxlint.

Então eu não descreveria Verter como "a mesma coisa com outro nome." É melhor pensar nisso como **mais uma resposta séria para a pergunta: como seria uma cadeia de ferramentas de próxima geração do Vue se recomeçassem?**

## Vize vs Vite+

[Vite+](https://viteplus.dev/) está em uma camada diferente.

Vite+ é um ponto de entrada unificado para o desenvolvimento web de forma mais ampla. Sua função é gerenciar a configuração em tempo de execução, gerenciamento de pacotes, desenvolvimento, verificação, teste, construção, empacotamento e execução de tarefas monorepo em um único fluxo de trabalho. Ela reúne ferramentas Vite, Vitest, Oxlint, Oxfmt, Rolldown, tsdown e ferramentas relacionadas.

Isso faz Vite+:

- **Agnóstico ao framework**
- Orientado a fluxos de trabalho
- mais amplo que o Vue

Vize é diferente porque é **específico da Vue**.

O Vite+ não tenta se tornar um compilador Vue ou um linter de templates Vue. Isso te dá um ponto de entrada unificado na web toolchain.
Vize pode se conectar a esse mundo. Na verdade, esse repositório já usa Vite+ para orquestração de workspace.

Então, isso não é realmente uma competição:

- **Vite+** = a shell geral da cadeia de ferramentas web
- **Vize** = o motor específico da Vue que pode viver dentro dessa carcaça

## Vize vs Oxlint

[Oxlint](https://oxc.rs/docs/guide/usage/linter) também está em uma camada diferente.

Oxlint é o linter de alto desempenho em JavaScript e TypeScript do ecossistema Oxc. Ele é excelente em regras gerais de JS/TS e fluxos de trabalho cada vez mais conscientes de tipos, mas sozinho não foi feito para substituir todos os diagnósticos conscientes de templates do Vue.

É aí que entra a Vize Patina.

A Patina foca em questões específicas de linting da Vue, como:

- Diretivas modelo
- Estrutura do SFC
- Convenções dos componentes
- verificações de acessibilidade nos modelos Vue

Então a diferença é simples:

- **Oxlint** lida com linting JS/TS de uso geral
- **Vize / Patina** trata linting específico da Vue

O novo `oxlint-plugin-vize` alfa existe justamente porque esses dois são complementares, e não redundantes.

## Então, onde fica a Vize?

Vize está na sobreposição entre várias categorias, mas não é redutível a nenhuma delas.

É:

- mais amplo do que as ferramentas oficiais da linguagem Vue
- mais amplo que `tsgo` projetos de aceleração como o Golar
- mais próximo em ambição de esforços alternativos full-stack como Verter
- complementar a ferramentas gerais de fluxo de trabalho como o Vite+
- complementar aos linters gerais JS/TS como Oxlint

Se eu tivesse que condensar em uma frase:

> Vize é uma tentativa independente nativa de Rust de unificar muito mais da cadeia de ferramentas do Vue do que as ferramentas oficiais de linguagem abrangem, ao mesmo tempo em que coopera com ferramentas mais amplas do ecossistema, em vez de substituí-las.

## Qual deles você deve escolher?

Isso depende do que você quer:

- Escolha **as Ferramentas de Linguagem do Vue** se você quer a pilha oficial de editor e verificação de digitação pronta para produção para o Vue hoje mesmo.
- Dê uma olhada **no Golar** se seu principal interesse for checar `typescript-go`tipos para linguagens embarcadas enquanto reutiliza ferramentas de idiomas oficiais.
- Dê uma olhada **no Verter** se quiser outra toolchain Vue full-stack ambiciosa, com uma tipagem forte e uma história de LSP rígida.
- Use **o Vite+** se quiser um ponto de entrada unificado para fluxo de trabalho geral e de desenvolvimento web.
- Use **Oxlint** se sua necessidade for JavaScript de alto desempenho e linting com TypeScript.
- Use **o Vize** se o que te empolga é a possibilidade de uma cadeia de ferramentas mais ampla nativa de Rust do Vue, que tente fazer compiladores, linting, formatação, verificação de tipos, ferramentas de editor, ferramentas de galeria e ferramentas de IA parecerem um só sistema.

Essa é a verdadeira diferença.
