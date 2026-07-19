---
title: Modo Vapor
description: Por que o Modo Vapor é importante para o Vize, e por que um caminho direto e detalhado do compilador muda mais do que o desempenho em tempo de execução.
---

<!-- Generated translation; source: blog/notes/2026-05-16-vapor-mode-and-the-next-vue-compiler-surface.md -->

# Modo Vapor

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

Modo Vapor é fácil de descrever de forma muito restrita.

A versão curta é: renderize componentes do Vue com um caminho mais direto e detalhado e menos overhead de DOM virtual.

Isso é verdade, mas isso não aborda a questão mais interessante das ferramentas.

Se o compilador se tornar mais direto, então a superfície do compilador se torna mais importante.

## Por que o Vapor Importa

A renderização tradicional Vue possui um modelo mental forte e maduro:

- compilar templates em funções de renderização
- criar nós virtuais
- Regiões dinâmicas diferenciais
- atualizar o DOM

Esse modelo é flexível e já experiente.

O Vapor pergunta o que acontece quando o compilador consegue gerar uma representação mais direta da interface reativa. Em vez de tratar o DOM virtual como a abstração central do tempo de execução, o compilador pode emitir operações que conectam a reatividade mais próxima das próprias atualizações do DOM.

Isso transfere a pressão da generalidade em tempo de execução para a precisão em tempo de compilação.

Para o Vize, isso é empolgante porque o Vize já é construído em torno da ideia de que uma cadeia de ferramentas do Vue deve entender profundamente o SFC antes que ele emita qualquer coisa.

## Um Tipo Diferente de Responsabilidade do Compilador

Quando a saída do compilador é mais direta, os erros ficam mais agudos.

O compilador precisa saber:

- quais ligações são reativas
- quais operações DOM são estáveis
- quais expressões precisam de getters
- quais props dinâmicos precisam de rotas de atualização
- quais slots e componentes exigem limites de tempo de execução
- quais escopos template são locais para loops, ramificações e slots

Em um modelo DOM virtual, alguma incerteza pode ser absorvida por diferência em tempo de execução.

Em um modelo mais direto no estilo Vapor, o compilador carrega mais da intenção. Isso significa que a qualidade da análise importa mais. O mapeamento de origem é o que importa mais. A cobertura instantânea importa mais.

Esse é exatamente o tipo de problema que a Vize foi criada para explorar.

## Vapor como um backend de primeira classe

A arquitetura do Vize trata os modos de saída do compilador como backends relacionados, e não como implementações não relacionadas.

A mesma estrutura SFC e análise de templates deve ser capaz de alimentar:

- Saída do compilador DOM
- Saída do compilador SSR
- Saída do compilador de vapor
- diagnósticos que explicam por que um construto é ou não suportado

Isso importa porque o vapor não deve se tornar um caso especial desconectado.

Se o suporte ao Vapor estiver no mesmo modelo de cadeia de ferramentas do suporte a DOM e SSR, o Vize pode comparar saídas, reutilizar snapshots e tornar os diagnósticos mais consistentes entre os modos.

## As Mudanças na Superfície de Depuração

O Modo Vapor também altera a experiência de depuração.

Quando a saída é mais direta, os desenvolvedores precisam confiar em:

- Ordenação de operações geradas
- Limites de dependência reativa
- Colocação de ouvintes no evento
- Semântica de atualização de prop de componentes
- Comportamento de limpeza de branch and loop
- hidratação ou compatibilidade SSR quando relevante

Isso não é apenas uma preocupação em tempo de execução. É uma preocupação de ferramentas.

Uma boa cadeia de ferramentas Vapor deve ajudar a responder:

- O que o compilador achava que era estático?
- O que ela achava que era dinâmico?
- De onde veio um caminho específico de atualização?
- Qual expressão fonte produziu essa operação gerada?
- Por que essa construção recuou ou falhou?

É aí que a abordagem de análise estática e testes de snapshots do Vize se torna útil.

## Desempenho sem perder a semântica

O Vapor é orientado para desempenho, mas o desempenho não pode vir às custas da semântica do Vue.

Os usuários não deveriam precisar memorizar uma segunda linguagem modelo só para usar o caminho mais rápido. O melhor resultado é que o compilador entende bem o código do Vue o suficiente para que a renderização direta pareça natural.

Isso exige:

- testes de compatibilidade com as expectativas normais do Vue
- Jogos do mundo real
- Diagnósticos precisos para padrões não suportados
- Mapeamento cuidadoso da fonte
- benchmarks que incluem grandes aplicações, não apenas exemplos de brinquedos

O objetivo não é "Vapor a qualquer custo."

O objetivo é um caminho de compilador rápido porque entende mais, não porque suporta menos silenciosamente.

## Por que isso combina com a Vize

Vize ainda é experimental. É exatamente por isso que o Vapor é um local natural para ele.

Uma cadeia de ferramentas independente pode explorar:

- Formas alternativas de saída do compilador
- Diagnósticos mais rigorosos
- Snapshots mais rápidos
- Modelagem direta de operação DOM
- Integração com análise de modelos consciente de tipos
- Explicações voltadas para IA sobre escolhas de compiladores

O ecossistema oficial precisa de estabilidade. Vize pode se mover mais rápido, testar agressivamente e aprender em público.

Esse é o relacionamento certo.

O Modo Vapor não é apenas mais uma caixa de seleção para o Vize. É um teste de estresse para toda a ideia de uma cadeia de ferramentas unificada do Vue.

Se parser, analisador, compilador, diagnósticos, snapshots e fixaturas do mundo real estiverem alinhados, então o Vapor se torna mais do que uma otimização em tempo de execução.

Isso se torna prova de que a cadeia de ferramentas entende o Vue profundamente o suficiente para gerar um futuro diferente para ele.
