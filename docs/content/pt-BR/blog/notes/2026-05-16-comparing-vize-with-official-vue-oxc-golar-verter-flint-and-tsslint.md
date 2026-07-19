---
title: Comparação de Ferramentas
description: Uma comparação prática da Vize e projetos próximos entre as ferramentas oficiais da Vue, Oxc, Golar, Verter, Flint e TSSLint.
---

<!-- Generated translation; source: blog/notes/2026-05-16-comparing-vize-with-official-vue-oxc-golar-verter-flint-and-tsslint.md -->

# Comparação de Ferramentas

<div class="blog-post-meta">
<span class="blog-meta-chip">
<span>
<span class="blog-meta-label">Publicado em</span>
<span class="blog-meta-value">16-05-2026</span>
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

Vize está próximo o suficiente de vários projetos para que a comparação seja inevitável.

Essa comparação é útil, mas somente se o eixo estiver claro. "Mais rápido" não é suficiente. "Ferrugem" não é suficiente. "Suporte Vue" não é suficiente.

A verdadeira questão é: **qual camada cada projeto quer possuir?**

![Relationship map showing Vize in the nearby tooling landscape, with reference-only, adjacent platform, used-by-Vize, and compare-only groups](/blog/vize-toolchain-map.svg)

## Mapa Rápido

| Projeto                     | Centro de gravidade                                                       | Como a Vize se relaciona com isso                                                                    |
| --------------------------- | ------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| Ferramentas oficiais do Vue | A linha base de produção para o compilador e ferramentas de linguagem Vue | Vize é independente e experimental, então deve tratar isso como ponto de referência                  |
| Oxc / Oxlint                | Infraestrutura geral de JavaScript e TypeScript                           | Vize pode reutilizar e cooperar com a Oxc enquanto possui semântica específica da Vue                |
| Golar                       | `typescript-go`verificação de tipos em linguagem embutida baseada em      | O Vize tem um escopo mais amplo da cadeia de ferramentas do Vue do que apenas a verificação de tipos |
| Verter                      | Compilador e cadeia de ferramentas alternativa de próxima geração do Vue  | O mais próximo em ambição, diferente em arquitetura e formato de produto                             |
| Flint                       | Linting JS/TS amigável, digitado com padrões fortes                       | Complementar para revestimento TS geral, não uma cadeia de ferramentas SFC do Vue                    |
| TSSLint                     | Linting nativo de TypeScript dentro do servidor de linguagem              | Ideia forte de linting semântico, mas não uma pilha completa de compilador/linter/galeria do Vue     |

## Ferramentas Oficiais da Vue

A pilha oficial importa primeiro.

[Vue Language Tools](https://github.com/vuejs/language-tools), `vue-tsc`, os pacotes compiladores do Vue e as integrações oficiais com editores são a base de produção. Quando o Vize discorda do comportamento oficial, esse desacordo não é automaticamente uma ideia nova e ousada. Na maioria das vezes, é uma correção necessária, uma implementação incompleta ou um ponto onde o Vize precisa de uma história de compatibilidade mais clara.

Isso não torna o Vize inútil.

Ela define o contrato.

O Vize pode experimentar uma arquitetura nativa de Rust mais unificada, mas ainda precisa se importar com a forma do código real do Vue, a saída real do compilador, diagnósticos reais e expectativas reais do editor. A stack oficial é o ponto de referência que mantém o experimento honesto.

## Oxc e Oxlint

[Oxc](https://oxc.rs/) é um projeto de infraestrutura de compiladores JavaScript e TypeScript de uso geral. [Oxlint](https://oxc.rs/docs/guide/usage/linter.html) é o linter de alto desempenho construído sobre esse mundo.

Vize não deve competir com o Oxc nas camadas JavaScript e TypeScript. Isso seria desperdício. O Oxc já oferece ao ecossistema um parser rápido, infraestrutura semântica, direção de formatação, direção de linter e um conjunto crescente de primitivas compartilhadas.

A questão do Vize é mais restrita e mais específica do Vue:

- O que é um arquivo `.vue` como um todo?
- Como os escopos template se conectam a bindings de script?
- Como diretivas, slots, props, emits, blocos de estilo e saída do compilador se relacionam?
- Como mapeamos diagnósticos de volta à fonte exata que os humanos editam?
- Como esses fluxos semânticos compilam, lint, formatam, verificam de tipos, LSP, Musea e IA?

O OXC pode ser a base geral do JS/TS. O Vize pode ser a cadeia de ferramentas específica do Vue que usa essa base sem achatar o Vue em "apenas blocos de script".

## Golar

[Golar](https://github.com/auvred/golar) é interessante porque leva `typescript-go` a sério para linguagens embarcadas.

Seu foco é a verificação de tipos, código virtual e integração `tsgo` . Para o Vue, isso naturalmente o coloca próximo do modelo oficial de linguagem central. Essa é uma forma boa e prática: reutilizar a maquinaria de código virtual do Vue e tornar o motor TypeScript mais rápido ou flexível.

Vize está tentando resolver um problema mais amplo.

A camada de verificação de tipos importa, mas não é o projeto todo. Vize quer que o parser, modelo semântico, compilador, linter, formatador, caminho nativo de verificação de tipo, LSP, galeria de componentes e superfícies voltadas para IA compartilhem mais do mesmo núcleo consciente do Vue.

Então a diferença não é "Golar está verificando tipos e Vize é mais rápido verificando tipo."

A diferença é:

- Golar é principalmente uma história de processamento TypeScript em linguagem embutida.
- Vize é uma história completa de toolchain Vue, onde a verificação de tipos é um dos consumidores do modelo de análise Vue.

## Verter

[Verter](https://github.com/pikax/verter) provavelmente é a comparação mais próxima filosoficamente.

Também está levantando uma grande questão: como seria uma cadeia de ferramentas do Vue de próxima geração se estivéssemos dispostos a repensar as camadas?

Isso é próximo da pergunta do Vize. Ambos os projetos se importam com o comportamento do compilador, ferramentas de linguagem, diagnósticos e uma experiência mais rigorosa do que uma bolsa de plugins não relacionados pode fornecer facilmente.

As diferenças estão em ênfase:

- Verter parece mais rigoroso e orientado ao serviço linguístico desde o início.
- Vize enfatiza um núcleo compartilhado nativo de Rust entre os fluxos de trabalho de compilação, lint, formatação, verificação, LSP, Musea e IA.
- O Vize também trata as ferramentas de galeria de componentes e sistemas de design como partes de primeira classe do ambiente frontend, e não como documentação separada e tardia.

Não vejo Verter como um inimigo. É mais um experimento sério em um campo que merece múltiplos experimentos.

## Flint

[Flint](https://www.flint.fyi/) é um tipo diferente de comparação.

É um linter em JavaScript e TypeScript com ênfase em padrões úteis, cache e linting digitado. Isso é valioso porque o ecossistema JS/TS tem um problema real: o linting apenas com sintaxe é rápido, mas incompleto, enquanto o linting semântico pode se tornar lento e operacionalmente caro.

Vize concorda com a premissa de que o feedback semântico deve ser prático, rápido e agradável.

Mas o Flint não está tentando ser um compilador, formatador, analisador de templates, galeria de componentes ou LSP específico do Vue SFC. É melhor entendido como uma direção geral de linting de alta qualidade.

A forma complementar é:

- Flint pode impulsionar a experiência de linting JS/TS adiante.
- O Vize pode impulsionar análises específicas do Vue.
- Um bom ambiente de frontend deve fazer essas camadas cooperarem, em vez de forçar todas as ferramentas a assumirem todas as preocupações.

## TSSLint

[TSSLint](https://marketplace.visualstudio.com/items?itemName=johnsoncodehk.vscode-tsslint) é importante porque trata o linting semântico do TypeScript como algo que pode ficar próximo ao servidor da linguagem TypeScript.

Essa ideia é convincente: se o verificador TypeScript já tem um projeto aberto, por que reconstruir o mundo em um processo linter separado só para responder a perguntas semânticas?

Vize tem um instinto semelhante, mas apontado para o Vue como um artefato multilíngue.

Para o Vize, a questão não é apenas "regras de lint podem reutilizar o estado do TypeScript?" É:

- A análise de templates pode reutilizar o mesmo modelo semântico do Vue que o compilador?
- As regras de fiapos do Vue que conscientizam de tipos podem fazer perguntas focadas sem pagar o custo total da reconstrução?
- Diagnósticos de editores, checagens em lote e loops de reparo de IA conseguem concordar no mesmo mapeamento de origem?
- O sistema consegue manter uma sessão de projeto viva tempo suficiente para amortizar o trabalho?

O TSSLint é um sinal forte de que o linting semântico quer se aproximar do estado existente da língua. Vize estende esse instinto para estruturas específicas de Vue.

## O que a Vize está tentando possuir

Vize não deveria possuir tudo.

Deve ser responsável pelos pontos onde o conhecimento específico do Vue deve ser coerente:

- Análise sintática SFC e estrutura de blocos
- Semântica do modelo
- Análise diretiva e de componentes
- Decisões de saída do compilador
- Diagnóstico de fiapos com Vue
- Mapeamento de origem dos artefatos gerados de volta para `.vue`
- Metadados componentes para Musea
- Diagnósticos legíveis por máquina para fluxos de trabalho de IA

Deve cooperar em outros lugares:

- use Oxc para análise sintática em JavaScript e TypeScript sempre que possível
- compare o comportamento com as ferramentas oficiais do Vue
- aprenda com Golar, TSSLint e Flint em loops de feedback conscientes de tipos
- fique atento ao Verter como mais um experimento de cadeia de ferramentas completa

## A Posição do Produto

O posicionamento mais limpo é o seguinte:

> Vize é uma toolchain independente, experimental e nativa de Rust do Vue que tenta fazer com que compilador, linter, formatador, verificador de tipos, LSP, galeria de componentes e diagnósticos voltados para IA parecessem um ambiente coerente.

Isso significa que Vize não é a resposta oficial.

É uma resposta experimental em alta velocidade.

O trabalho agora é tornar essa resposta útil em projetos reais, reduzir a diferença com o comportamento oficial e manter a arquitetura afiada o suficiente para que o experimento valha a pena.
