---
title: Testes e Agentes
description: Por que testes com muitos snapshots, fixaturas do mundo real e verificações determinísticas importam mais quando agentes fazem parte do ciclo de desenvolvimento.
---

<!-- Generated translation; source: blog/notes/2026-05-16-testing-agentic-coding-and-trust.md -->

# Testes e Agentes

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

A Codificação Agential muda o papel dos testes.

Quando um humano escreve um pequeno patch, testes nos indicam se o patch quebrou algo.

Quando um agente pode reescrever grandes trechos de código, os testes também se tornam a linguagem que usamos para dizer ao agente o que significa "bom".

Isso torna os testes mais importantes, não menos.

## Testes são a memória do projeto

Agentes são bons em raciocínio local, mas um projeto é maior que o prompt atual.

Uma cadeia de ferramentas acumulou decisões:

- O que o diagnóstico deve dizer
- onde os intervalos de origem devem apontar
- Como o código gerado deve ser
- quais casos de extremidade do Vue são suportados
- quais projetos reais precisam continuar compilando
- quais falsos positivos são inaceitáveis

Os testes preservam essas decisões.

Sem testes, toda mudança agential é forçada a redescobrir o projeto do zero. Com testes, o projeto pode resistir. Pode dizer: esse comportamento importa, essa saída é intencional, essa mensagem de erro faz parte da experiência do usuário.

## Testes de snapshot são especialmente úteis

O Vize usa muitos snapshots porque as blockchains produzem resultados estruturados que os humanos precisam inspecionar:

- Saída do compilador
- Saída de formatador
- Diagnóstico de linter
- TypeScript virtual
- Locais de diagnóstico mapeados por fonte
- Metadados Musea gerados
- Construa artefatos de projetos de fixtures

Snapshots não substituem asserções. Eles são uma forma de tornar o comportamento amplo revisável.

Isso importa para a Codificação Agentica porque agentes podem criar grandes diferenciais rapidamente. Um bom conjunto de snapshots torna esses diffs visíveis de uma forma que os humanos possam revisar. Ele transforma "algo mudou em algum lugar do compilador" em "essa saída de renderização mudou exatamente neste caso."

Essa é uma superfície de avaliação muito melhor.

## Determinismo é o contrato

Fluxos de trabalho agentes precisam de ferramentas determinísticas.

Se os testes forem instáveis, o agente não consegue dizer se o adesivo ajudou. Se a ordem de saída mudar entre as execuções, os snapshots se tornam ruído. Se os diagnósticos dependem do estado ambiente da máquina, o IC se torna uma loteria.

Então a Vize se importa com detalhes entediantes:

- Ordenação estável de saída
- IDs diagnósticos estáveis
- Aberturas de fonte estáveis
- Forma de código gerada estável
- Configuração estável de luminárias
- Diretórios Scratch isolados

Determinismo não é apenas para CI. É o que permite que humanos e agentes compartilhem o mesmo ciclo de retroalimentação.

## Jogos do Mundo Real Mantêm o Sistema Honesto

Testes unitários são necessários, mas as ferramentas do Vue vivem em projetos reais.

Projetos reais têm:

- Grafos de importação incomuns
- Layouts do gerenciador de pacotes
- Arquivos gerados
- Convenções macro
- Pré-processadores de estilo
- árvores componentes enormes
- Padrões antigos ao lado de padrões novos

Por isso, o Vize continua testando contra fixatures e snapshots do mundo real. O objetivo é não afirmar prontidão para produção cedo demais. O objetivo é encontrar cada aresta afiada que só aparece fora de um aplicativo de amostragem perfeita.

Esse tipo de verificação exaustiva demora a se desenvolver, mas é o caminho do experimento para a ferramenta real.

## Testes são uma conversa com a comunidade

O feedback da comunidade não é apenas comentários rastreadores.

Também é:

- um projeto real que não compila
- um diagnóstico que aponta para o intervalo errado
- um falso positivo que bloqueia a adoção
- um abismo de desempenho em um repositório que ninguém previu
- um padrão de produção que a cadeia de ferramentas não entendia

Cada um desses relatórios deve se tornar um ponto fixo, um teste de regressão ou um parâmetro.

É assim que o feedback vira memória. É assim que uma ferramenta experimental se torna mais séria com o tempo.

## Agentes precisam de loops menores e melhores

A pior configuração de testes para agentes é um comando enorme e lento que falha no final com uma mensagem pouco clara.

A melhor configuração fornece feedback em camadas:

- Testes unitários rápidos para invariantes locais
- Testes snapshot para revisão de saída
- Testes de fixture para comportamento de framework
- Testes de integração focados para limites de ferramentas
- Matrizes CI para plataformas e construções de produção

Agentes podem usar essa escada. Os humanos também podem.

Essa é uma das razões pelas quais a Vize continua investindo em ferramentas de teste e consolidação de scripts. Um bom projeto deve tornar o cheque correto fácil de executar, fácil de entender e fácil de escalar quando o risco aumenta.

## A confiança é conquistada repetidamente

Nenhuma toolchain se torna confiável só porque seu README diz "rápido" ou "correto".

A confiança é conquistada toda vez:

- Um diagnóstico é preciso
- Uma correção não danifica o código próximo
- Uma mudança instantânea é explicável
- Um projeto do mundo real continua passando
- O CI detecta algo antes do lançamento
- Um agente pode iterar sem perder a thread

Por isso, testar não é uma missão secundária para a Vize.

Faz parte do produto.

Na era da IA, as melhores ferramentas não serão aquelas que geram mais código. Eles serão os que podem gerar, validar, explicar e rejeitar código em ciclos determinísticos e apertados.

Os testes são onde esses ciclos se tornam reais.
