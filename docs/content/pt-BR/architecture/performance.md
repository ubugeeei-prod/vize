---
title: Desempenho
---

<!-- Generated translation; source: architecture/performance.md -->

# Desempenho

> **⚠️ Trabalho em andamento:** O Vize está em desenvolvimento ativo e ainda não está pronto para uso em produção. Os números de benchmark vêm de builds de desenvolvimento e podem mudar.

O Vize alcança melhorias significativas de desempenho em relação ao compilador padrão baseado em JavaScript do Vue ao aproveitar as abstrações de custo zero e o multithreading nativo da Rust. Velocidade não é algo agradável — é um pré-requisito para a experiência do desenvolvedor.

## Ambiente de Benchmark

Os números históricos abaixo foram capturados em uma estação de trabalho local. Para números
reprodutíveis hospedados por CI adequados para notas de lançamento e atualizações de documentação, use o
[Blacksmith benchmark snapshot](./performance-blacksmith) gerado pelo fluxo de trabalho Tool Benchmark.

|             |                                                |
| ----------- | ---------------------------------------------- |
| **Máquina** | MacBook Pro (M2 Max, 12 núcleos, 96 GB de RAM) |
| **OS**      | macOS 15.3.2 (Darwin 24.3.0)                   |
| **Node.js** | v24.14.0                                       |
| **Vite**    | v8.0.0 (Rolamento)                             |
| **Vue**     | v3.6.0-beta.10                                 |

## Benchmark: 15.000 arquivos SFC

Compilando **15.000 arquivos SFC do Vue** (36,9 MB no total):

|                                  | @vue/compilador-sfc | Vize  | Aceleração |
| -------------------------------- | ------------------- | ----- | ---------- |
| **Fio Único**                    | 9,35s               | 3,47s | **2,7x**   |
| **Multi Thread**                 | 4,08s               | 353ms | **11,6x**  |
| **compilador-sfc ST vs Vize MT** | 9,35s               | 353ms | **26,0x**  |

A melhoria single-threaded vem das abstrações de custo zero da Rust (sem GC, sem aquecimento JIT, layout de memória compatível com cache). A melhoria multithread vem do pool de threads da Rayon, que rouba trabalho, e escala quase linearmente com a contagem de núcleos da CPU.

### Comportamento Nativo de Escalonamento por Lote

| Arquivos | Lote Vize (1 thread) | Lote Vize (12 threads) | Aceleração paralela |
| -------- | -------------------- | ---------------------- | ------------------- |
| 100      | 25ms                 | 3ms                    | 8,5x                |
| 1,000    | 243ms                | 26ms                   | 9,4x                |
| 5,000    | 1,25s                | 128ms                  | 9,7x                |
| 15,000   | 3,75s                | 373ms                  | 10,1x               |

Esses números nativos de lote incluem leituras de arquivos. Pequenos lotes são dominados por custos fixos; lotes maiores se estabelecem em torno de 10x a velocidade paralela nessa máquina de 12 núcleos.

## Por que ferrugem?

### Abstrações de custo zero

O modelo de propriedade da Rust elimina pausas na coleta de lixo. O compilador processa nós AST por meio da alocação arena (`vize_carton`), evitando alocações por nó de heap. Isso significa:

- **Sem pausas no GC** — Em compiladores baseados em V8, a coleta de lixo pode causar picos imprevisíveis de latência. Vize não tem nenhuma overhead de GC.
- **Sem aquecimento JIT** — o compilador JIT do V8 precisa de tempo para otimizar os caminhos quentes. Vize roda em velocidade máxima desde a primeira instrução.
- **Desempenho previsível** — A compilação antecipada do Rust significa que o desempenho é consistente entre as execuções, não depende das heurísticas de otimização do V8.

### Multithreading Nativo

O Vize utiliza [Rayon](https://docs.rs/rayon) para compilação paralela de dados. Cada arquivo SFC é compilado independentemente, tornando a carga de trabalho embaraçosamente paralela. O agendador de roubo de trabalho da Rayon garante a utilização ótima dos núcleos:

```rust
// Simplified: parallel compilation of all .vue files
files.par_iter().map(|file| {
    let arena = Bump::new();
    let ast = parse(file, &arena);
    let analyzed = analyze(ast, &arena);
    compile(analyzed, &arena)
}).collect()
```

A abordagem de roubo de trabalho significa que, se um arquivo for significativamente maior que os outros, threads ociosas vão roubar trabalho da fila da thread ocupada, mantendo um balanceamento de carga quase perfeito.

### Disposição eficiente da memória

O layout de estruturas e os discriminantes de enum da Rust são compactos. A representação AST em `vize_relief` é amigável ao cache, reduzindo gargalos de largura de banda de memória:

- **Discriminantes de enum** — Os enums de ferrugem são dimensionados no menor tipo que se encaixe no discriminante. Um `NodeKind` com 20 variantes usa um único byte, não uma string alocada ao heap.
- **Empacotamento de struct** — O Rust reordena automaticamente os campos de sstrut para alinhamento ideal, minimizando bytes de preenchimento (padding bytes).
- **Sem cabeçalhos de objeto** — Ao contrário dos objetos JavaScript (que carregam chains de protótipo, mapas de propriedades e ponteiros de classe ocultos), structs Rust são dados puros com zero overhead.

### Sem overhead de tempo de execução

Diferente dos compiladores baseados em JavaScript que rodam na V8, o Vize compila diretamente para código nativo. Não há aquecimento JIT, nem coletor de lixo, nem contenção de loop de eventos. O binário do compilador é um único executável estaticamente ligado que inicia e roda em velocidade máxima.

## Escolhas de Arquitetura para Desempenho

### Alocação de Arenas

`vize_carton` fornece um alocador de bump para nós AST usando [bumpalo](https://docs.rs/bumpalo). Isso significa:

- **A alocação é O(1)** — Basta avançar um ponteiro. Sem percurso livre de listas, sem gerenciamento de fragmentação.
- **A deslocação é O(1)** — Descarte toda a arena de uma vez quando a compilação estiver concluída. Sem sobrecarga de realocação por nó.
- **A localidade da memória é excelente** — os nós são empacotados contíguos na memória, maximizando os impactos no cache L1/L2 durante a travessia da árvore.

Essa é uma vantagem fundamental sobre o coletor de lixo geracional do V8, que precisa rastrear objetos acessíveis e compactar a memória periodicamente.

### Tokenizador de Streaming

O tokenizer do `vize_armature`processa a entrada como um fluxo de bytes, evitando a necessidade de construir arrays intermediários de tokens. O analisador consome tokens preguiçosamente — cada token é produzido sob demanda e imediatamente consumido. Isso reduz o uso máximo de memória e melhora o comportamento do cache.

### Estágio de cordas

Strings comuns (nomes de diretivas, nomes de atributos, nomes de tags HTML) são internadas via tabelas de hash `compact_str` e perfeitas (`phf`). Isso significa:

- A comparação de strings é comparação de ponteiros (O(1)) em vez de comparação caractere por caractere (O(n))
- Cadeias duplicadas compartilham uma única alocação
- As buscas de hash para strings conhecidas são calculadas em tempo de compilação

### Compilação Incremental

O plugin Vite (`@vizejs/vite-plugin`) usa cache em nível de arquivo. Apenas arquivos modificados são recompilados durante o desenvolvimento, minimizando a latência do HMR. A chave de cache é o hash do conteúdo do arquivo, garantindo que arquivos não alterados nunca sejam recompilados.

## Benchmark: Linter — patina vs eslint-plugin-vue

- \*Linting 15.000 arquivos Vue SFC\*\*:

|           | eslint-plugin-vue (ST) | Pátina Vize (ST) | Aceleração | eslint-plugin-vue (MT) | Pátina Vize (MT) | Aceleração | **eslint ST vs Vize MT** |
| --------- | ---------------------- | ---------------- | ---------- | ---------------------- | ---------------- | ---------- | ------------------------ |
| **Tempo** | 45,08s                 | 4,02s            | **11,2x**  | 16,38s                 | 784ms            | **20,9x**  | **57,5x**                |

Correr `vp run --workspace-root bench:lint` para se reproduzir.

### Perfil de fiapos consciente do tipo

O linting consciente de tipos é intencionalmente perfilado nas fases em que o custo tende a se agrupar: análise sintática SFC, análise
Croquis, geração virtual de TypeScript, coleta de consultas de templates e sondas Corsa. Quando
múltiplas regras de type-aware apoiadas por templates são ativadas, o Patina coleta a expressão do template e
consultas Promise template em uma única caminhada AST antes da fase de sonda Corsa. A coleção de consultas também compartilha
análise de expressões OXC para verificações unsafe-template e floating-Promise, de modo que uma expressão template
não paga custo de análise duplicado quando ambas as regras estão ativadas.

Faça `vize lint --profile --preset opinionated src` para ver essas linhas em um projeto local. O relatório de perfil
também inclui uma seção rigorosa de auditoria que verifica a cobertura de tempo de parede, o tempo acumulado
trabalhador, os acertos de limiar lento e os períodos internos capturados antes de listar arquivos quentes e operações internas de
. Linhas de arquivo quente mostram participação e throughput por estágio, e linhas de operação sinalizam spans dominantes de
ou picos máximos/médios.

## Benchmark: Formatter — glifo vs Pretier

Formatação **de 15.000 arquivos Vue SFC**:

|           | Mais bonita (CLI) | Glifo Vize (ST) | Aceleração | Glifo Vize (MT) | **Cli mais bonito vs Vize MT** |
| --------- | ----------------- | --------------- | ---------- | --------------- | ------------------------------ |
| **Tempo** | 101,20s           | 2,97s           | **34,1x**  | 835ms           | **121,2x**                     |

Correr `vp run --workspace-root bench:fmt` para se reproduzir.

## Benchmark: Type Checker — cânone vs vue-tsc

Verificação de tipos **de 500 arquivos SFC gerados no Vue** com o caminho de diagnóstico atual respaldado pela Corsa:

|           | vue-tsc (ST)   | Cânone Vize (ST) | Aceleração | vue-tsc (MT)   | Cânone Vize (MT) | Aceleração | **vue-tsc ST vs Vize MT** |
| --------- | -------------- | ---------------- | ---------- | -------------- | ---------------- | ---------- | ------------------------- |
| **Tempo** | 4,38s          | 511ms            | **8,6x**   | 4,41s          | 493ms            | **8,9x**   | **8,9x**                  |
| **Taxa**  | 114 arquivos/s | 979 arquivos/s   |            | 113 arquivos/s | 1.0k arquivos/s  |            |                           |

> **Nota:** O canhão Vize ainda está em desenvolvimento inicial e o caminho de diagnóstico apoiado pela Corsa ainda está alcançando a fidelidade vue-tsc. Essas medições refletem a implementação nativa atual com CLI primeiro, com um recurso de reserva por sessão de projeto, e mudarão à medida que a cobertura e a paridade de diagnóstico melhoram.

Execute `node bench/check.ts 500` após `cargo build --release -p vize` para reproduzir esse benchmark rápido.

### Perfil do verificador de tipos

O dispositivo de perfil 500-SFC mantém a maior parte do tempo de parede dentro do comando CLI Corsa, enquanto o caminho rápido de importação remove o custo anterior de análise OXC para arquivos sem especificadores Vue:

| Métrica                         | Antes   | Atualidade |
| ------------------------------- | ------- | ---------- |
| `canon.import.rewrite.vue`      | 26,77ms | 2,45ms     |
| Maior TS Virtual gerado         | 15.401B | 14.414B    |
| Tempo total na parede do perfil | 1,88s   | 668ms      |
| Fase de diagnóstico da Corsa    | 1,67s   | 482ms      |
| Análise do CLI Corsa            | N/A     | 10,41ms    |

A fase de `virtual project` do lado Rust — análise SFC por arquivo, análise Croquis
geração de Virtual TS e reescrita de importação — é distribuída pelo thread
pool do rayon dentro de `VirtualProject::register_paths`. Cada arquivo `.vue` é independente
uma vez que as opções do workspace são resolvidas, então um único lote paraleliza
de forma limpa. Em um dispositivo de 1.000 SFC, a fase cai de ~71 ms para ~25 ms antes mesmo de
Corsa ser ativada.

### Luminária e2e com muitos diagnósticos

`bench/check.ts` também mede o aplicativo `tests/_fixtures/_git/npmx.dev` quando o aparelho está presente. Isso captura o caminho de mapeamento de diagnóstico em um dispositivo real de aplicação:

| Jogos               | Arquivos SFC fonte | Arquivos virtuais | Diagnósticos | Cânone Vize |
| ------------------- | ------------------ | ----------------- | ------------ | ----------- |
| npmx.dev aplicativo | 134                | 226               | 1,053        | 1,94s       |

O perfil atual desse aparelho mantém a análise diagnóstica do CLI em ~7ms. A maior parte do tempo agora está no comando CLI da Corsa. A autoimportação de stubs do framework em um único arquivo ambiente também reduziu o maior arquivo Virtual TS gerado de cerca de 275KB para 144KB.

## Benchmark: Vite Plugin — @vizejs/vite-plugin vs @vitejs/plugin-vue

Build do Vite com **1.000 importações do SFC Vue** (todas importadas em uma única entrada):

|                         | @vitejs/plugin-vue | @vizejs/vite-plugin | Aceleração |
| ----------------------- | ------------------ | ------------------- | ---------- |
| **Tempo de Construção** | 957ms              | 479ms               | **2.0x**   |

> Nota: `@vizejs/vite-plugin` substitui apenas a etapa de compilação do Vue SFC — a diferença de desempenho vem inteiramente dessa parte. A resolução de dependências, construção de grafos de módulos, agrupamento (Rolldown) e todos os outros internos do Vite são idênticos aos `@vitejs/plugin-vue`. Para performance pura em compilações, veja o [Compiler benchmark](#benchmark-15000-sfc-files) acima. `@vizejs/vite-plugin` pré-compila `.vue` arquivos com entusiasmo usando compilação multithreaded nativa, que também permite um HMR mais rápido.

Correr `vp run --workspace-root bench:vite` para se reproduzir.
