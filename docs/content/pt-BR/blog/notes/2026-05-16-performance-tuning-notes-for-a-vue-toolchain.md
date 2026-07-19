---
title: Ajuste de Performance
description: Lições práticas de performance ao construir uma caixa de ferramentas Vue onde parsing, alocação, paralelismo e loops de feedback são todos importantes.
---

<!-- Generated translation; source: blog/notes/2026-05-16-performance-tuning-notes-for-a-vue-toolchain.md -->

# Ajuste de Performance

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

Ajuste de desempenho em uma cadeia de ferramentas frontend não é um truque só.

Não é "reescrever no Rust" e depois esperar os gráficos subirem. É uma longa série de pequenas decisões concretas sobre para onde o tempo vai, com que frequência a memória se move, quanto trabalho é duplicado e se a arquitetura permite que as melhorias se acumulem.

Esta nota é um compartilhamento de conhecimento sobre as coisas que o Vize continua otimizando.

![Feedback loop diagram showing source files, native analysis, snapshots, actions, and shipping confidence](/blog/feedback-loop.svg)

## Meça o Loop Completo

Benchmarks de compiladores são úteis, mas não são toda a experiência para desenvolvedores.

Uma cadeia de ferramentas do Vue possui vários ciclos de feedback:

- Compilação de um único arquivo em um servidor de desenvolvimento
- Montagem em produção completa
- Linting de muitos arquivos
- formatação de muitos arquivos
- Verificação de tipos de arquivos virtuais gerados
- Diagnóstico do editor enquanto o usuário digita
- Verificações de CI em aplicações reais
- Patches gerados por IA sendo validados repetidamente

O loop mais lento nem sempre é o mais óbvio.

Uma função que parece rápida isoladamente ainda pode ser prejudicial se rodar em todos os estágios. Uma pequena alocação ainda pode importar se isso acontecer para cada token, cada nó AST, todo diagnóstico e cada segmento gerado.

Por isso, o Vize trata o desempenho como uma propriedade da toolchain, e não apenas como uma propriedade do compilador.

## Evite Trabalhos Duplicados

A otimização mais confiável é não fazer o trabalho duas vezes.

Em uma configuração fragmentada, o mesmo arquivo `.vue` pode ser analisado separadamente por:

- O compilador
- O Linter
- O Formador
- O Verificador de Tipos
- A integração do editor
- O pipeline de documentação de componentes

Isso é caro, mas o problema mais profundo é arquitetônico. Se cada ferramenta construir seu próprio entendimento do arquivo, a afinação de desempenho se torna local e limitada.

Vize foi projetado em torno de uma estrutura compartilhada:

- Analise uma vez quando possível
- manter os limites dos blocos SFC estáveis
- Estrutura de modelo de reutilização em compiladores e diagnósticos
- Deixe a análise semântica alimentar múltiplos consumidores
- evitar regenerar o TypeScript virtual a menos que as entradas tenham sido alteradas

A melhor otimização geralmente é um limite de posse melhor.

## Alocação é uma característica, não um detalhe

Ferramentas frontend processam muitos objetos pequenos: tokens, nós, spans, strings, escopos, diagnósticos, fragmentos de código gerados.

Se esses objetos forem alocados casualmente, a cadeia de ferramentas paga por eles em todos os lugares.

Vize exerce muita pressão sobre o comportamento de alocação:

- Armazenamento no estilo arena para dados de compiladores de curta duração
- internamento de string onde identificadores ou nomes repetidos importam
- Espaços compactos em vez de substrings copiadas
- fatias emprestadas onde a propriedade é desnecessária
- IDs internos estáveis em vez de grandes estruturas clonadas

O objetivo não é tornar o código inteligente por si só.

O objetivo é tornar o caminho quente entediante: menos alocações, menos cópias, menos falhas de cache, menos motivos para o alocador se tornar parte do perfil.

## Paralelismo precisa de forma

Paralelismo não é "ligar fios".

Funciona melhor quando o problema tem limites claros:

- muitos arquivos independentes
- Agregação determinística
- Ordenação de saída previsível
- sem mutação global compartilhada
- caches e sessões limitadas

A compilação, linting e varreduras de fixture Vue frequentemente apresentam uma forma paralela natural em nível de lima. Mas a verificação de tipos e os fluxos de trabalho do editor são mais sutis porque dependem do estado do projeto.

Então o Vize separa as perguntas:

- Esse trabalho em nível de arquivo pode rodar de forma independente?
- Essa etapa precisa de uma sessão de projeto residente?
- A ordem de saída é visível para o usuário?
- Os diagnósticos são estáveis em diferentes contagens de threads?
- O paralelismo aumenta a pressão de memória o suficiente para apagar a vitória?

Saída rápida, mas instável, não é suficiente. O trabalho de performance precisa preservar a confiança.

## O mapeamento de fonte pode se tornar um caminho quente

As ferramentas do Vue frequentemente geram código intermediário.

Isso significa que todo bom diagnóstico precisa de um caminho de volta:

- gerou TypeScript para o template original
- Código de renderização gerado para o código-fonte SFC
- saída de estilo ou script transformado para o bloco original
- IDs de módulos virtuais voltando para arquivos reais

Se o mapeamento de fontes for lento ou impreciso, toda a cadeia de ferramentas sofre. O usuário vê os diagnósticos no lugar errado. Loops de reparo da IA ficam com coordenadas ruins. Os testes ficam frágeis.

Portanto, o mapeamento de origem merece a mesma atenção de desempenho que a análise sintática:

- Armazenamento se estende de forma compacta
- evitar normalização repetida de caminhos
- Mantenha os metadados gerados de segmentos pequenos
- Casos de fronteira de teste com snapshots
- cargas de trabalho pesadas em diagnóstico de perfil, não apenas caminhos de compilação bem-sucedidos

Diagnósticos são superfícies de produto. O desempenho deles importa.

## Projetos Reais Superam o Conforto Sintético

Microbenchmarks são úteis ao responder a uma pergunta focada.

Mas uma cadeia de ferramentas se torna honesta quando é executada contra projetos reais.

Projetos reais incluem:

- Layouts de dependência ímpares
- grandes SFCs
- Padrões legados
- Código gerado automaticamente
- Diretrizes incomuns
- Convenções de plugins
- Aliases de caminho
- Casos extremos específicos de plataforma

É por isso que a Vize continua investindo em varridas de fixtures e builds snapshots. O objetivo não é coletar contagem impressionante de testes. O objetivo é expor os cliffs de desempenho que só aparecem quando o código está bagunçado, da mesma forma que o código de produção é bagunçado.

## Desempenho é uma característica do produto

Velocidade muda o comportamento.

Se as verificações são lentas, as pessoas as fazem com menos frequência.
Se a formatação estiver lenta, o save on-format se torna irritante.
Se o linting consciente do tipo estiver lento, as equipes desativam as regras.
Se o IC é lento, os mantenedores fazem as mudanças em lote e revisam com menos cuidado.
Se a validação da IA é lenta, os agentes fazem saltos maiores e mais arriscados.

Ferramentas rápidas tornam fluxos de trabalho mais rigorosos práticos.

Esse é o verdadeiro argumento de desempenho para a Vize. O objetivo não é apenas um número de referência melhor. O objetivo é fazer o caminho rígido parecer o padrão do caminho.

Quando compilar, lint, formatar, checagem de tipos e diagnósticos se tornam rápidos o suficiente para rodar sem cerimônia, a qualidade deixa de ser um evento especial.

Virou o jeito normal de trabalhar.
